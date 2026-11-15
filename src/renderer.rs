use std::sync::Arc;

use hecs::{Entity, World};
use nalgebra::vector;
use wgpu::{Adapter, Backends, CommandEncoderDescriptor, CompositeAlphaMode, CurrentSurfaceTexture, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, LoadOp, Operations, PowerPreference, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, TextureViewDescriptor};
use winit::window::Window;

use crate::{math::Vector2, sprite::{Sprite, create_pipeline}};

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub struct Renderer {
    _instance: Instance,
    _adapter: Adapter,
    surface: Surface<'static>,
    pub gpu: Arc<Gpu>,
    surface_config: SurfaceConfiguration,
    sprite_pipeline: RenderPipeline,

    window: Arc<Window>,
} impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN | Backends::GL,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Renderer failed to create surface from window");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .expect("Renderer failed o request adapter");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                required_limits: Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                ..Default::default()
            })
            .await
            .expect("Renderer failed to find device and queue");

        let gpu = Arc::new(Gpu { device, queue });

        let size = window.inner_size();
        let (min_1_width, min_1_height) = (size.width.max(1), size.height.max(1));

        let mut surface_config = surface
            .get_default_config(&adapter, min_1_width, min_1_height)
            .expect("Renderer failed to get default config from surface");

        surface_config.view_formats = vec![surface_config.format.add_srgb_suffix()];
        surface_config.alpha_mode = CompositeAlphaMode::Opaque;

        surface.configure(&gpu.device, &surface_config);

        let sprite_pipeline = create_pipeline(&gpu, surface_config.format);

        Self {
            _instance: instance,
            surface,
            _adapter: adapter,
            gpu,
            surface_config,
            sprite_pipeline,
            window,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.gpu.device, &self.surface_config);
    }

    pub fn size_vec(&self) -> Vector2 {
        let extent = self.window.inner_size();
        vector![extent.width as f32, extent.height as f32]
    }

    pub fn request_redraw(&mut self) {
        self.window.request_redraw();
    }
}

pub fn render(world: &mut World, global: Entity) {
    let rdr: &mut Renderer = &mut world.get::<&mut Renderer>(global).unwrap();
    let output = match rdr.surface.get_current_texture() {
        CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
        CurrentSurfaceTexture::Suboptimal(surface_texture) => {
            surface_texture
        }
        CurrentSurfaceTexture::Timeout
        | CurrentSurfaceTexture::Occluded
        | CurrentSurfaceTexture::Validation => {
            return;
        }
        CurrentSurfaceTexture::Outdated => {
            rdr.surface.configure(&rdr.gpu.device, &rdr.surface_config);
            return;
        }
        CurrentSurfaceTexture::Lost => {
            panic!("Lost device");
        }
    };
    let view = output.texture.create_view(&TextureViewDescriptor::default());
    let mut encoder = rdr.gpu.device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let mut render_pass: wgpu::RenderPass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        render_pass.set_pipeline(&rdr.sprite_pipeline);

        let mut query = world.query::<(&mut Sprite,)>();
        for (sprite,) in query.iter() {
            sprite.draw(&mut render_pass)
        }
    }

    rdr.gpu.queue.submit(std::iter::once(encoder.finish()));
    rdr.gpu.queue.present(output);
    rdr.request_redraw();
}
