use crate::draw_state;
use crate::mesh;
use crate::na;
use crate::physics;
use crate::wgputil;

use wgpu::util::DeviceExt;

use itertools::Itertools;

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniforms {
    view: na::Matrix4,
    position: na::Vector4,
    projection: na::Matrix4,
}

fn view_from_orientation(orientation: na::Matrix4) -> na::Matrix4 {
    let mut r = orientation;
    r.set_column(2, &(-1.0 * r.column(2))); // wgpu is right handed
    r.transpose() // inverse of orthogonal matrix is transpose
}

impl CameraUniforms {
    fn from_components(camera: &Camera, body: &physics::RigidBody) -> Self {
        CameraUniforms {
            view: view_from_orientation(body.orientation.to_matrix()),
            position: body.position,
            projection: na::Matrix4::new_perspective(
                camera.aspect,
                camera.fovy,
                camera.znear,
                camera.zfar,
            ),
        }
    }
}

pub struct MeshBuffers {
    num_vertices: u32,
    _tetrahedra_buffer: wgpu::Buffer,
    transform_buffer: wgpu::Buffer,
    draw_state_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TransformUniforms {
    linear: na::Matrix4,
    translation: na::Vector4,
}

impl TransformUniforms {
    fn from_body(body: &physics::RigidBody) -> Self {
        Self {
            linear: body.orientation.to_matrix(),
            translation: body.position,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DrawStateUniforms {
    pub outline: na::Vector4,
    pub hollow: u32,
    pub padding: [u32; 3],
}

impl DrawStateUniforms {
    fn from_draw_state(state: &draw_state::DrawState) -> Self {
        Self {
            outline: state.outline,
            hollow: state.hollow as u32,
            padding: [0, 0, 0],
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Mix {
    a: u32,
    b: u32,
    p1: u32, // padding, since array elements in std140 layout are 16-btye aligned
    p2: u32, // padding
}

pub fn mixtable_bytes() -> Vec<u8> {
    // A tetrahedron has 4 vertices, and each one of those vertices may be "above",
    // in, or "below" the view-hyperplane. This gives 81 possible situations, resulting
    // in either nothing (or a mere point or line), a triangle, a quad, or the entire
    // tetrahedron being rendered.
    // Each tetrahedron is sent to the vertex shader 7 times in order ot create a
    // triangle strip at least long enough to cover rending the full tetrahedron (6
    // vertices on the strip), plus a leading degenerate triangle to avoid weird
    // inter-tetrahedral triangles.
    // Each of the 7 invocations corresponds results in a new vertex on the strip
    // being created from some combination of the vertices in the tetrahedron, based
    // on which of 81 possible strips is relevant to the tetrahedron.
    [[0, 1, 2]]
        .repeat(4)
        .into_iter()
        .multi_cartesian_product()
        .map(|signs| {
            // sort each vertex into above, below, or in plane
            let mut inplane: Vec<u32> = Vec::new();
            let mut above: Vec<u32> = Vec::new();
            let mut below: Vec<u32> = Vec::new();
            for (j, s) in signs.iter().enumerate() {
                match s {
                    0 => below.push(j as u32),
                    1 => inplane.push(j as u32),
                    2 => above.push(j as u32),
                    _ => (),
                }
            }

            // create a mix for each vertex or combination of vertices that intersects the plane
            let mut combinations: Vec<Mix> = Vec::new();
            for vi in &inplane {
                combinations.push(Mix {
                    a: *vi,
                    b: *vi,
                    p1: 0,
                    p2: 0,
                });
            }
            for avi in above {
                for bvi in &below {
                    combinations.push(Mix {
                        a: avi,
                        b: *bvi,
                        p1: 0,
                        p2: 0,
                    });
                }
            }

            let mut row: [Mix; 7] = [Mix::default(); 7];
            // If more than 3 point in the plane, set up the combinations that will
            // make up the vertices of the triangles formed by those points
            if combinations.len() >= 3 {
                // The first triangle is always the same - it includes a duplicate
                // point in the beginning, to bridge it to the previous tetrahedron
                // in the triangle strip.
                row[0] = combinations[0];
                row[1] = combinations[0];
                row[2] = combinations[1];
                row[3] = combinations[2];

                if combinations.len() == 3 {
                    // If there are exactly 3 points in the plane, fill the rest of
                    // of the strip with duplicates to make degenerate triangles
                    row[4] = combinations[2];
                    row[5] = combinations[2];
                    row[6] = combinations[2];
                } else if combinations.len() == 4 {
                    // Otherwise, the result is at least a quad - add another
                    // meaningful point
                    row[4] = combinations[3];
                    if inplane.len() == 4 {
                        // If all vertices were exactly in the plane, the result
                        // should be the full tetrahedron - fill out the triangle
                        // strip with the rest of its vertices
                        row[5] = combinations[0];
                        row[6] = combinations[1];
                    } else {
                        // Otherwise, it was just a quad after all - fill with
                        // duplicates to induce degeneracy
                        row[5] = combinations[3];
                        row[6] = combinations[3];
                    }
                }
            }
            row
        })
        .flat_map(|x| bytemuck::bytes_of(&x).to_vec())
        .collect::<Vec<u8>>()
}

pub struct MeshRenderer {
    render_pipeline: wgpu::RenderPipeline,

    mesh_bind_group_layout: wgpu::BindGroupLayout,

    _mixtable_uniform_buffer: wgpu::Buffer,
    mixtable_uniform_bind_group: wgpu::BindGroup,

    camera_uniforms_buffer: wgpu::Buffer,
    camera_uniforms_bind_group: wgpu::BindGroup,
}

impl MeshRenderer {
    pub fn new(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let mixtable_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });
        let camera_uniforms_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });
        let mesh_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let render_pipeline = wgputil::create_pipeline(
            device,
            surface_config,
            true,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/slice.vert"
            )),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/slice.frag"
            )),
            &[
                &mixtable_uniform_bind_group_layout,
                &camera_uniforms_bind_group_layout,
                &mesh_bind_group_layout,
            ],
        );

        let mixtable_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: mixtable_bytes().as_slice(),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let mixtable_uniform_bind_group = wgputil::bind_group(
            device,
            &mixtable_uniform_bind_group_layout,
            &mixtable_uniform_buffer,
        );

        let camera_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<CameraUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_uniforms_bind_group = wgputil::bind_group(
            device,
            &camera_uniforms_bind_group_layout,
            &camera_uniforms_buffer,
        );

        Self {
            render_pipeline,

            mesh_bind_group_layout,

            _mixtable_uniform_buffer: mixtable_uniform_buffer,
            mixtable_uniform_bind_group,

            camera_uniforms_buffer,
            camera_uniforms_bind_group,
        }
    }

    pub fn update_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &mut hecs::World,
        camera_entity: hecs::Entity,
    ) {
        if let Ok((camera, body)) =
            world.query_one_mut::<(&Camera, &physics::RigidBody)>(camera_entity)
        {
            queue.write_buffer(
                &self.camera_uniforms_buffer,
                0,
                bytemuck::bytes_of(&CameraUniforms::from_components(camera, body)),
            );
        }

        let mut new_meshes = Vec::new();
        for (entity, (body, mesh, draw_state)) in world
            .query::<(&physics::RigidBody, &mesh::Mesh4, &draw_state::DrawState)>()
            .iter()
        {
            if let Ok(mesh_buffers) = world.get::<&MeshBuffers>(entity) {
                queue.write_buffer(
                    &mesh_buffers.transform_buffer,
                    0,
                    bytemuck::bytes_of(&TransformUniforms::from_body(body)),
                );
                queue.write_buffer(
                    &mesh_buffers.draw_state_buffer,
                    0,
                    bytemuck::bytes_of(&DrawStateUniforms::from_draw_state(draw_state)),
                );
            } else {
                let tetrahedra_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(mesh.get_buffer_data().as_slice()),
                        usage: wgpu::BufferUsages::STORAGE,
                    });

                let transform_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::bytes_of(&TransformUniforms::from_body(body)),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });

                let draw_state_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::bytes_of(&DrawStateUniforms::from_draw_state(
                            draw_state,
                        )),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.mesh_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: tetrahedra_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: transform_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: draw_state_buffer.as_entire_binding(),
                        },
                    ],
                    label: None,
                });

                new_meshes.push((
                    entity,
                    MeshBuffers {
                        num_vertices: (mesh.num_tetrahedra * 7) as u32,
                        _tetrahedra_buffer: tetrahedra_buffer,
                        transform_buffer,
                        draw_state_buffer,
                        bind_group,
                    },
                ));
            }
        }

        for (entity, mesh_buffers) in new_meshes {
            world.insert_one(entity, mesh_buffers).unwrap();
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        world: &hecs::World,
    ) {
        // Instantiate the query here due to lifetime parameter of set_bind_group
        // forcing the bind_group parameter to live as long as the render pass
        // itself, i.e. bind_group parameter inherits the lifetime of the query,
        // which would get dropped at the end of the for loop otherwise while the
        // render pass lives on. So, the query must be declared before the render
        // pass.
        let mut mesh_buffers_query = world.query::<&MeshBuffers>();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.2,
                        g: 0.2,
                        b: 0.25,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.mixtable_uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_uniforms_bind_group, &[]);

        for (_, mesh_buffers) in mesh_buffers_query.iter() {
            render_pass.set_bind_group(2, &mesh_buffers.bind_group, &[]);
            render_pass.draw(0..mesh_buffers.num_vertices, 0..1);
        }
    }
}
