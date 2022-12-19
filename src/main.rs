mod actor;
mod collision;
mod constraints;
mod contact;
mod draw_state;
mod ga;
mod gjk;
mod input;
mod joint;
mod mesh;
mod mesh_renderer;
mod mpr;
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

async fn run(
    event_loop: EventLoop<()>,
    window: winit::window::Window,
    mut world: hecs::World,
    player_entity: hecs::Entity,
) {
    let mut renderer = renderer::Renderer::new(&window);
    let mut constraints = constraints::Constraints::new();
    let mut input_state = input::InputState::default();

    let mut cursor_mode = winit::window::CursorGrabMode::None;
    window.set_cursor_grab(cursor_mode).unwrap();

    let dt: f32 = 1.0 / 120.0;
    let mut remaining: f32 = 0.0;
    let mut last_frame_time = instant::Instant::now();

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
                WindowEvent::MouseInput { button, state, .. } => {
                    input_state.mouse_click(*button, *state == ElementState::Pressed);
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
                let frame_start_time = instant::Instant::now();
                let elapsed = frame_start_time - last_frame_time;
                last_frame_time = frame_start_time;
                remaining += (elapsed.as_nanos() as f64 / 1e9) as f32;

                while remaining > 0.0 {
                    actor::update_actor(&mut constraints, &mut world, &input_state, player_entity);
                    collision::do_collisions(&mut constraints, &mut world);
                    physics::apply_physics(dt, &mut constraints, &mut world);
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

fn build_world() -> (hecs::Entity, hecs::World) {
    let mut world = hecs::World::new();

    let player_entity = world.spawn((
        actor::Actor {
            move_thrust: 3000.0,
            look_torque: 500.0,
            grab_state: actor::GrabState::Not,
        },
        mesh_renderer::Camera {
            fovy: 1.0,
            znear: 0.1,
            zfar: 100.0,
        },
        sprite_renderer::Sprite {
            scale: na::vec2(1.0, 1.0),
            position: na::Vector2::zeros(),
            tint: na::vec4(0.0, 0.0, 0.0, 1.0),
        },
        physics::RigidBody {
            position: na::vec4(0.0, 0.0, -4.0, 0.0),
            linear_damping: 0.9,
            angular_damping: 0.9,
            gravity: 0.0,
            ..Default::default()
        }
        .with_mass(100.0),
    ));
    {
        let floor_mesh = mesh::Mesh4::cube().transformed(&na::Affine4::from_pos(
            na::Vector4::zeros(),
            na::Matrix4::identity(),
            na::vec4(10.0, 10.0, 10.0, 10.0),
        ));
        world.spawn((
            physics::RigidBody {
                position: na::vec4(0.0, -7.0, 0.0, 0.0),
                gravity: 0.0,
                ..Default::default()
            }
            .with_mass(f32::INFINITY),
            collision::Collider::from_mesh4(&floor_mesh),
            floor_mesh,
            draw_state::DrawState {
                contacts: 0,
                hollow: false,
            },
        ));
    }
    world.spawn((
        physics::RigidBody {
            position: na::vec4(0.0, 0.0, 0.0, 0.0),
            ..Default::default()
        },
        mesh::Mesh4::cube(),
        collision::Collider::from_mesh4(&mesh::Mesh4::cube()),
        draw_state::DrawState {
            contacts: 0,
            hollow: false,
        },
    ));
    world.spawn((
        physics::RigidBody {
            position: na::vec4(0.0, 1.1, 0.0, 0.0),
            angular_velocity: na::vec4(0.7, 0.0, 0.0, 0.7).wedge(na::vec4(0.0, 0.0, 0.7, 0.0))
                + na::vec4(0.7, 0.7, 0.0, 0.0).wedge(na::vec4(0.0, 0.0, 0.0, 0.5)),
            gravity: 0.0,
            ..Default::default()
        },
        mesh::Mesh4::cube(),
        collision::Collider::from_mesh4(&mesh::Mesh4::cube()),
        draw_state::DrawState {
            contacts: 0,
            hollow: false,
        },
    ));
    (player_entity, world)
}

fn main() {
    let event_loop = EventLoop::new();
    let (player_entity, world) = build_world();

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        pollster::block_on(run(event_loop, window, world, player_entity));
    }

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowBuilderExtWebSys;

        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");

        let canvas = web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.get_element_by_id("canvas"))
            .map(|el| el.unchecked_into::<web_sys::HtmlCanvasElement>())
            .expect("Canvas not found");
        let window = WindowBuilder::new()
            .with_canvas(Some(canvas))
            .build(&event_loop)
            .unwrap();

        wasm_bindgen_futures::spawn_local(run(event_loop, window, world, player_entity));
    }
}
