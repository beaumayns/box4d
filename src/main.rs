mod actor;
mod collision;
mod draw_state;
mod ga;
mod gjk;
mod input;
mod joint;
mod mesh;
mod mesh_renderer;
mod na;
mod physics;
mod renderer;
mod sprite_renderer;
mod texture;
mod wgputil;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::ga::Wedge;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut cursor_mode = winit::window::CursorGrabMode::Locked;
    window.set_cursor_grab(cursor_mode).unwrap();

    let mut renderer = renderer::Renderer::new(&window);
    let mut world = hecs::World::new();
    let mut input_state = input::InputState::default();

    let player_entity = world.spawn((
        actor::Actor {
            move_thrust: 3000.0,
            look_torque: 500.0,
            grab_state: actor::GrabState::Not,
        },
        mesh_renderer::Camera {
            aspect: window.inner_size().width as f32 / window.inner_size().height as f32,
            fovy: 1.0,
            znear: 0.1,
            zfar: 100.0,
        },
        sprite_renderer::Sprite {
            image: "assets/images/crosshair.png".into(),
            scale: na::vec2(1.0, 1.0),
            position: na::Vector2::zeros(),
            tint: na::vec4(0.0, 0.0, 0.0, 1.0),
        },
        physics::RigidBody {
            position: na::vec4(0.0, 0.0, -4.0, 0.0),
            linear_damping: 0.9,
            angular_damping: 0.9,
            ..Default::default()
        }
        .with_mass(100.0),
    ));
    world.spawn((
        physics::RigidBody {
            position: na::vec4(-1.0, 0.0, 0.0, 0.0),
            angular_velocity: na::vec4(0.7, 0.0, 0.0, 0.7).wedge(na::vec4(0.0, 0.0, 0.7, 0.0))
                + na::vec4(0.7, 0.7, 0.0, 0.0).wedge(na::vec4(0.0, 0.0, 0.0, 0.5)),
            ..Default::default()
        },
        mesh::Mesh4::cube(),
        collision::Collider::from_mesh4(&mesh::Mesh4::cube()),
        draw_state::DrawState {
            outline: na::Vector4::zeros(),
            hollow: false,
        },
    ));
    world.spawn((
        physics::RigidBody {
            position: na::vec4(1.0, 0.0, 0.0, 0.0),
            ..Default::default()
        },
        mesh::Mesh4::cube(),
        collision::Collider::from_mesh4(&mesh::Mesh4::cube()),
        draw_state::DrawState {
            outline: na::Vector4::zeros(),
            hollow: false,
        },
    ));

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
                    renderer.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    renderer.resize(**new_inner_size);
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
            Event::DeviceEvent {
                event: DeviceEvent::Button { button, state },
                ..
            } => {
                input_state.mouse_click(button, state == ElementState::Pressed);
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
                    actor::update_actor(&mut world, &input_state, player_entity);
                    physics::apply_physics(dt, &mut world);
                    remaining -= dt;
                }

                renderer.update_buffers(&mut world, player_entity);
                match renderer.render(&world) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
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
