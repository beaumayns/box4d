mod ga;
mod input;
mod mesh;
mod na;
mod rigidbody;
mod texture;
mod wgputil;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::ga::Wedge;

use crate::na::{vec2, vec4};

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniforms {
    view: na::Matrix4,
    position: na::Vector4,
    projection: na::Matrix4,
}

struct Camera {
    body: rigidbody::RigidBody,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

fn view_from_orientation(orientation: na::Matrix4) -> na::Matrix4 {
    let mut r = orientation;
    r.set_column(2, &(-1.0 * r.column(2))); // wgpu is right handed
    r.transpose() // inverse of orthogonal matrix is transpose
}

impl Camera {
    fn uniforms(&self) -> CameraUniforms {
        CameraUniforms {
            view: view_from_orientation(self.body.orientation.to_matrix()),
            position: self.body.position,
            projection: na::Matrix4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TransformUniforms {
    linear: na::Matrix4,
    translation: na::Vector4,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteUniforms {
    scale: na::Vector2,
    position: na::Vector2,
    tint: na::Vector4,
}

struct State {
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    depth_texture: texture::Texture,

    crosshair_sprite_texture_bind_group: wgpu::BindGroup,
    _crosshair_texture: texture::Texture,
    crosshair_sprite_uniforms_bind_group: wgpu::BindGroup,
    _crosshair_sprite_uniforms_buffer: wgpu::Buffer,
    render_pipeline_sprite: wgpu::RenderPipeline,

    mesh: mesh::Mesh4,
    mesh_body: rigidbody::RigidBody,
    camera: Camera,

    _mixtable_uniform_buffer: wgpu::Buffer,
    mixtable_uniform_bind_group: wgpu::BindGroup,

    camera_uniforms_buffer: wgpu::Buffer,
    camera_uniforms_bind_group: wgpu::BindGroup,

    _tetrahedra_buffer: wgpu::Buffer,
    transform_buffer: wgpu::Buffer,
    mesh_bind_group: wgpu::BindGroup,

    render_pipeline_4d: wgpu::RenderPipeline,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .expect("Failed to create device");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &surface_config);

        let depth_texture = texture::Texture::create_depth_texture(
            &device,
            &surface_config,
            wgpu::TextureFormat::Depth32Float,
        );

        let crosshair_image = image::open("assets/images/crosshair.png").unwrap();
        let crosshair_image_rgba = crosshair_image.as_rgba8().unwrap();
        let crosshair_texture = texture::Texture::from_image(&device, &queue, crosshair_image_rgba);
        let crosshair_sprite_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        let crosshair_sprite_texture_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &crosshair_sprite_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&crosshair_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&crosshair_texture.sampler),
                    },
                ],
                label: None,
            });

        let crosshair_sprite_uniforms_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&SpriteUniforms {
                    scale: vec2(
                        crosshair_image_rgba.dimensions().0 as f32 / surface_config.width as f32,
                        crosshair_image_rgba.dimensions().1 as f32 / surface_config.height as f32,
                    ),
                    position: vec2(0.0, 0.0),
                    tint: vec4(0.0, 0.0, 0.0, 0.0),
                }),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let crosshair_sprite_uniforms_bind_group_layout =
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

        let crosshair_sprite_uniforms_bind_group = wgputil::bind_group(
            &device,
            &crosshair_sprite_uniforms_bind_group_layout,
            &crosshair_sprite_uniforms_buffer,
        );

        let render_pipeline_sprite = wgputil::create_pipeline(
            &device,
            &surface_config,
            true,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/sprite.vert"
            )),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/sprite.frag"
            )),
            &[
                &crosshair_sprite_uniforms_bind_group_layout,
                &crosshair_sprite_texture_bind_group_layout,
            ],
        );

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
                ],
                label: None,
            });


        let mesh: mesh::Mesh4 = mesh::Mesh4::cube();
        // Initialize the cube to be orbiting the origin while revolving
        let mesh_body = rigidbody::RigidBody {
            angular_velocity: vec4(0.7, 0.0, 0.0, 0.7).wedge(vec4(0.0, 0.0, 0.7, 0.0))
                + vec4(0.7, 0.7, 0.0, 0.0).wedge(vec4(0.0, 0.0, 0.0, 0.5)),
            ..Default::default()
        };

        // Place the camera 4 units away from the origin.
        // Default orientation will have it looking at the origin
        let camera = Camera {
            body: rigidbody::RigidBody {
                position: vec4(0.0, 0.0, -4.0, 0.0),
                linear_damping: 0.9,
                angular_damping: 0.9,
                ..Default::default()
            },
            aspect: surface_config.width as f32 / surface_config.height as f32,
            fovy: 1.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mixtable_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: mesh::mixtable_bytes().as_slice(),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let mixtable_uniform_bind_group = wgputil::bind_group(
            &device,
            &mixtable_uniform_bind_group_layout,
            &mixtable_uniform_buffer,
        );

        let camera_uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&camera.uniforms()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_uniforms_bind_group = wgputil::bind_group(
            &device,
            &camera_uniforms_bind_group_layout,
            &camera_uniforms_buffer,
        );

        let tetrahedra_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(mesh.get_buffer_data().as_slice()),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&TransformUniforms::default()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let mesh_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &mesh_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: tetrahedra_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: transform_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        let render_pipeline_4d = wgputil::create_pipeline(
            &device,
            &surface_config,
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

        Self {
            size,
            surface,
            device,
            queue,
            surface_config,
            depth_texture,

            crosshair_sprite_texture_bind_group,
            _crosshair_texture: crosshair_texture,
            crosshair_sprite_uniforms_bind_group,
            _crosshair_sprite_uniforms_buffer: crosshair_sprite_uniforms_buffer,
            render_pipeline_sprite,

            mesh,
            mesh_body,
            camera,

            _mixtable_uniform_buffer: mixtable_uniform_buffer,
            mixtable_uniform_bind_group,

            camera_uniforms_buffer,
            camera_uniforms_bind_group,

            _tetrahedra_buffer: tetrahedra_buffer,
            transform_buffer,
            mesh_bind_group,
            render_pipeline_4d,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);

            self.camera.aspect =
                self.surface_config.width as f32 / self.surface_config.height as f32;
        }
    }

    fn apply_input(&mut self, input_state: &input::InputState) {
        let orientation = self.camera.body.orientation.to_matrix();

        const MOVE_THRUST: f32 = 20.0;
        const LOOK_TORQUE: f32 = 5.0;

        let right = orientation.column(0).normalize();
        let up = orientation.column(1).normalize();
        let forward = orientation.column(2).normalize();
        let ana = orientation.column(3).normalize();

        if input_state.up {
            self.camera.body.force += up * MOVE_THRUST;
        }
        if input_state.down {
            self.camera.body.force -= up * MOVE_THRUST;
        }
        if input_state.right {
            self.camera.body.force += right * MOVE_THRUST;
        }
        if input_state.left {
            self.camera.body.force -= right * MOVE_THRUST;
        }
        if input_state.forward {
            self.camera.body.force += forward * MOVE_THRUST;
        }
        if input_state.back {
            self.camera.body.force -= forward * MOVE_THRUST;
        }
        if input_state.ana {
            self.camera.body.force += ana * MOVE_THRUST;
        }
        if input_state.kata {
            self.camera.body.force -= ana * MOVE_THRUST;
        }

        let (yaw_plane, pitch_plane, roll_plane) = match input_state.hyperlook {
            false => (forward.wedge(right), up.wedge(forward), right.wedge(up)),
            true => (forward.wedge(ana), up.wedge(ana), right.wedge(ana)),
        };
        self.camera.body.torque += LOOK_TORQUE * input_state.yaw * yaw_plane;
        self.camera.body.torque += LOOK_TORQUE * input_state.pitch * pitch_plane;
        if input_state.roll_left {
            self.camera.body.torque += 3.0 * LOOK_TORQUE * roll_plane;
        }
        if input_state.roll_right {
            self.camera.body.torque += -3.0 * LOOK_TORQUE * roll_plane;
        }
    }

    fn update(&mut self, input_state: &input::InputState, dt: f32) {
        self.apply_input(input_state);
        self.mesh_body.update(dt);
        self.camera.body.update(dt);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::bytes_of(&TransformUniforms {
                linear: self.mesh_body.orientation.to_matrix(),
                translation: self.mesh_body.position,
            }),
        );
        self.queue.write_buffer(
            &self.camera_uniforms_buffer,
            0,
            bytemuck::bytes_of(&self.camera.uniforms()),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline_4d);
            render_pass.set_bind_group(0, &self.mixtable_uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_uniforms_bind_group, &[]);
            render_pass.set_bind_group(2, &self.mesh_bind_group, &[]);
            // Each tetrahedron must be rendered 7 times, to cover all possible
            // combinations of vertices (arranged in a triangle strip) that may
            // intersect the hyperplane of the camera.
            render_pass.draw(0..(self.mesh.num_tetrahedra * 7) as u32, 0..1);

            render_pass.set_pipeline(&self.render_pipeline_sprite);
            render_pass.set_bind_group(0, &self.crosshair_sprite_uniforms_bind_group, &[]);
            render_pass.set_bind_group(1, &self.crosshair_sprite_texture_bind_group, &[]);
            render_pass.draw(0..4u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut cursor_mode = winit::window::CursorGrabMode::Locked;
    window.set_cursor_grab(cursor_mode).unwrap();

    let mut state = futures::executor::block_on(State::new(&window));

    let mut input_state = input::InputState::default();

    let dt: f32 = 1.0 / 120.0;
    let mut remaining: f32 = 0.0;
    let mut last_frame_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::NewEvents(_) => input_state.new_events(),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(keycode),
                            state: key_state,
                            ..
                        },
                    ..
                } => {
                    let pressed = *key_state == ElementState::Pressed;
                    input_state.keyboard_event(*keycode, pressed);
                    match *keycode {
                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        VirtualKeyCode::Tab if pressed => {
                            cursor_mode = match cursor_mode {
                                winit::window::CursorGrabMode::Locked => {
                                    winit::window::CursorGrabMode::None
                                }
                                winit::window::CursorGrabMode::None => {
                                    winit::window::CursorGrabMode::Locked
                                }
                                _ => cursor_mode,
                            };
                            window.set_cursor_grab(cursor_mode).unwrap();
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta: (x, y) },
                ..
            } => {
                if cursor_mode == winit::window::CursorGrabMode::Locked {
                    input_state.mouse_moved(x, y);
                }
            }
            Event::RedrawRequested(_) => {
                // Constant-time physics updates - accumulate the elapsed time
                // since previous frame, and spend it in fixed chunks doing physics
                // updates
                let frame_start_time = std::time::Instant::now();
                let elapsed = frame_start_time - last_frame_time;
                last_frame_time = frame_start_time;
                remaining += (elapsed.as_nanos() as f64 / 1e9) as f32;

                while remaining > 0.0 {
                    state.update(&input_state, dt);
                    remaining -= dt;
                }

                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}
