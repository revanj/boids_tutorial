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
        let renderer = world.entity(global).unwrap().get::<&Renderer>().unwrap();
        let screen_size = renderer.size();
        let mut rng = rand::rng(); 
        drop(renderer);
        
        for _ in 0..100 {
            let renderer = 
	            world.entity(global).unwrap().get::<&Renderer>().unwrap();
            let sprite = Sprite::new_arrow(&renderer.gpu);
            drop(renderer);
            let x: f32 = rng.random();
            let y: f32 = rng.random();
            let pos = (nalgebra::vector![x, y] - vector![0.5, 0.5])
	            .component_mul(&screen_size);
            let transform = Transform(Mat4::new_scaling(30f32));
            let position = Position(pos);
            let rand_angle = rng.random::<f32>() * 2f32 * PI;
            let velocity = Velocity(
	            vector![f32::cos(rand_angle), f32::sin(rand_angle)]);
            world.spawn((sprite, position, velocity, transform));
        }
    }

    fn update(world: &mut World, global: Entity) {
        let left_margin = -350f32;
        let right_margin = 350f32;
        let bottom_margin = -350f32;
        let top_margin = 350f32;
        let avoid_factor = 0.01f32;
        let matching_factor = 0.05f32;
        let centering_factor = 0.0005f32;
        let turn_factor = 0.2f32;
        let max_speed = 2.0f32;
        let min_speed = 1.0f32;
        let visual_range = 50.0f32;
        let protected_range = 10.0f32;

        {
            let mut query = world.query::<(Entity, &mut Velocity, &mut Position)>();
            let arrow_etts: Vec<Entity> = query.iter().map(|x| x.0).collect();
            let mut view = query.view();

            for i in 0..arrow_etts.len() {
	            let mut vel_avoid = vector![0f32, 0f32];
	            let mut vel_avg = vector![0f32, 0f32];
                let mut pos_avg = vector![0f32, 0f32];
                let mut count = 0;
                
                for j in i..arrow_etts.len() {
                    if i == j {continue;}
                    let [this, other] = view.get_disjoint_mut([arrow_etts[i], arrow_etts[j]]);
                    let [(_, this_vel, this_pos), (_, other_vel, other_pos)] = [this.unwrap(), other.unwrap()];
                    if (this_pos.0 - other_pos.0).norm() > visual_range { continue; }
                    pos_avg += other_pos.0;
                    vel_avg += other_vel.0;
                    count += 1;
                    
                    if (this_pos.0 - other_pos.0).norm() > protected_range { continue; }
                    vel_avoid += this_pos.0 - other_pos.0;
                }
                
                vel_avg /= count as f32;
                pos_avg /= count as f32;
                let (_, this_vel, this_pos) = view.get_mut(arrow_etts[i]).unwrap();
                let vel_delta = vel_avg - this_vel.0;
                let pos_delta = pos_avg - this_pos.0;

                if count != 0 {
                    this_vel.0 += vel_delta * matching_factor;
                    this_vel.0 += pos_delta * centering_factor;
                }
                this_vel.0 += vel_avoid * avoid_factor;

				if this_pos.0.x < left_margin {
                    this_vel.0.x += turn_factor;
                }        
                if this_pos.0.x > right_margin {
                    this_vel.0.x -= turn_factor;
                }
                if this_pos.0.y < bottom_margin {
                    this_vel.0.y += turn_factor;
                }
                if this_pos.0.y > top_margin {
                    this_vel.0.y -= turn_factor;
                }
                let speed = this_vel.0.norm();
                if speed > max_speed {
                    this_vel.0 = (this_vel.0 / speed) * max_speed;
                }
                if speed < min_speed {
                    this_vel.0 = (this_vel.0 / speed) * min_speed;
                }
                
                this_pos.0 += this_vel.0;
            }
        }
        for (pos, heading, tf) in world.query::<(&Position, &Velocity, &mut Transform)>().iter() {
            tf.0[(0, 3)] = pos.0.x;
            tf.0[(1, 3)] = pos.0.y;
            let scaled_heading = heading.0.normalize() * 15f32;
            // basis x is (heading.y -heading.x)
            // basis y is (heading.x heading.y)
            tf.0.set_column(0, &vector![scaled_heading.y, -scaled_heading.x, 0f32, 0f32]);
            tf.0.set_column(1, &vector![scaled_heading.x, scaled_heading.y, 0f32, 0f32]); 
        }
    }
}

trait Game {
    fn new() -> Self;
    fn init(world: &mut World, global: Entity);
    fn update(world: &mut World, global: Entity);
}

struct MousePosition(Vector2);
struct Transform(Mat4);

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
                let renderer: &mut Renderer = &mut self.world.get::<&mut Renderer>(self.global).unwrap();
                renderer.resize(size.width, size.height);
            }
            WindowEvent::CursorMoved { device_id: _, position } => {

            },
            _ => ()
        }
    }
}

