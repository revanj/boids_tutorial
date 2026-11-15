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

    window: Arc<Window>,
} impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {

        Self {
            window,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
    }

    pub fn size(&self) -> Vector2 {
        let extent = self.window.inner_size();
        vector![extent.width as f32, extent.height as f32]
    }

    pub fn request_redraw(&mut self) {
        self.window.request_redraw();
    }
}

pub fn render(world: &mut World, global: Entity) {

}
