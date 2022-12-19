use crate::na;
use crate::texture;
use crate::wgputil;

use wgpu::util::DeviceExt;

use crate::na::{vec2, vec4};

pub struct Sprite {
    pub scale: na::Vector2,
    pub position: na::Vector2,
    pub tint: na::Vector4,
}

pub struct SpriteBuffers {
    uniforms_buffer: wgpu::Buffer,
    _texture: texture::Texture,
    texture_dimensions: na::Vector2,
    bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteUniforms {
    scale: na::Vector2,
    position: na::Vector2,
    tint: na::Vector4,
}

pub struct SpriteRenderer {
    render_pipeline: wgpu::RenderPipeline,

    bind_group_layout: wgpu::BindGroupLayout,
}

impl SpriteRenderer {
    pub fn new(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let render_pipeline = wgputil::create_pipeline(
            device,
            surface_config,
            false,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/sprite.vert"
            )),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/sprite.frag"
            )),
            &[&bind_group_layout],
            &[],
        );

        Self {
            render_pipeline,

            bind_group_layout,
        }
    }

    pub fn update_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_dimensions: winit::dpi::PhysicalSize<u32>,
        world: &mut hecs::World,
    ) {
        let mut new_sprites = Vec::new();
        for (entity, sprite) in world.query::<&Sprite>().iter() {
            if let Ok(sprite_buffers) = world.get::<&SpriteBuffers>(entity) {
                queue.write_buffer(
                    &sprite_buffers.uniforms_buffer,
                    0,
                    bytemuck::bytes_of(&SpriteUniforms {
                        scale: vec2(
                            sprite.scale[0] * sprite_buffers.texture_dimensions[0]
                                / screen_dimensions.width as f32,
                            sprite.scale[1] * sprite_buffers.texture_dimensions[1]
                                / screen_dimensions.height as f32,
                        ),
                        position: sprite.position,
                        tint: sprite.tint,
                    }),
                );
            } else {
                let image_bytes = include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/images/crosshair.png"
                ));
                let image = image::load_from_memory(image_bytes).unwrap();
                let image_rgba = image.as_rgba8().unwrap();
                let texture = texture::Texture::from_image(device, queue, image_rgba);

                let uniforms_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::bytes_of(&SpriteUniforms {
                            scale: vec2(
                                sprite.scale[0] * image_rgba.dimensions().0 as f32
                                    / screen_dimensions.width as f32,
                                sprite.scale[1] * image_rgba.dimensions().1 as f32
                                    / screen_dimensions.height as f32,
                            ),
                            position: vec2(0.0, 0.0),
                            tint: vec4(0.0, 0.0, 0.0, 1.0),
                        }),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: uniforms_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&texture.sampler),
                        },
                    ],
                    label: None,
                });

                new_sprites.push((
                    entity,
                    SpriteBuffers {
                        uniforms_buffer,
                        _texture: texture,
                        texture_dimensions: vec2(
                            image_rgba.dimensions().0 as f32,
                            image_rgba.dimensions().1 as f32,
                        ),
                        bind_group,
                    },
                ));
            }
        }

        for (entity, sprite_buffers) in new_sprites {
            world.insert_one(entity, sprite_buffers).unwrap();
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        world: &hecs::World,
    ) {
        let mut sprite_buffers_query = world.query::<&SpriteBuffers>();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Sprite Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        for (_, sprite_buffers) in sprite_buffers_query.iter() {
            render_pass.set_bind_group(0, &sprite_buffers.bind_group, &[]);
            render_pass.draw(0..4u32, 0..1);
        }
    }
}
