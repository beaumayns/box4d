use crate::mesh_renderer;
use crate::sprite_renderer;
use crate::texture;

pub struct Renderer {
    pub size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    depth_texture: texture::Texture,

    pub mesh_renderer: mesh_renderer::MeshRenderer,
    pub sprite_renderer: sprite_renderer::SpriteRenderer,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter =
            futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }))
            .expect("failed to get adaptor");

        let (device, queue) = futures::executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .expect("failed to get device");

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

        let mesh_renderer = mesh_renderer::MeshRenderer::new(&device, &surface_config);
        let sprite_renderer = sprite_renderer::SpriteRenderer::new(&device, &surface_config);

        Self {
            size,
            surface,
            device,
            queue,
            surface_config,
            depth_texture,

            mesh_renderer,
            sprite_renderer,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);

            self.depth_texture = texture::Texture::create_depth_texture(
                &self.device,
                &self.surface_config,
                wgpu::TextureFormat::Depth32Float,
            );
        }
    }

    pub fn update_buffers(&mut self, world: &mut hecs::World, camera_entity: hecs::Entity) {
        self.mesh_renderer
            .update_buffers(&self.device, &self.queue, world, camera_entity);
        self.sprite_renderer
            .update_buffers(&self.device, &self.queue, self.size, world);
    }

    pub fn render(&mut self, world: &hecs::World) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        self.mesh_renderer
            .render(&mut encoder, &view, &self.depth_texture.view, world);
        self.sprite_renderer.render(&mut encoder, &view, world);

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }
}
