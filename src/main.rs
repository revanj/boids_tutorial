mod math;
mod renderer;
mod sprite;

use std::{f32::consts::PI, sync::Arc};
use hecs::{Entity, World};
use nalgebra::vector;
use rand::RngExt;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowId}};

use crate::{math::{Camera, Mat4, Vector2}, renderer::{Renderer, render}, sprite::Sprite};

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut engine = Engine::<BoidsTutorial>::new();
    event_loop.run_app(&mut engine).unwrap();
}

struct Position(Vector2);
struct Velocity(Vector2);

struct BoidsTutorial {}
impl Game for BoidsTutorial {
    fn new() -> Self { Self {} }
    fn init(world: &mut World, global: Entity) {
    }

    fn update(world: &mut World, global: Entity) {
    }
}

trait Game {
    fn new() -> Self;
    fn init(world: &mut World, global: Entity);
    fn update(world: &mut World, global: Entity);
}

struct MousePosition(Vector2);

struct Engine<G: Game> {
    finished_init: bool,
    global: Entity,
    world: World,
    _game: G,
    
} impl<G: Game> Engine<G> {
    pub fn new() -> Self {
        let mut world = World::new();
        let global = world.spawn(());

        Self {
            finished_init: false,
            global,
            world,
            _game: G::new(),
        }
    }
}

impl<G: Game> ApplicationHandler for Engine<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        
        let mut renderer = pollster::block_on(Renderer::new(window));
        renderer.request_redraw();

        self.world.insert(self.global, (renderer,)).unwrap();

        G::init(&mut self.world, self.global);
        self.finished_init = true;

    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                if self.finished_init {
                    G::update(&mut self.world, self.global);

                    { // sync sprite transform with the GPU 
                        
                    }

                    render(&mut self.world, self.global);
                }
            },
            WindowEvent::Resized(size) 
            if self.finished_init && size.width != 0 && size.height != 0 => {

            }
            WindowEvent::CursorMoved { device_id: _, position } => {

            },
            _ => ()
        }
    }
}

