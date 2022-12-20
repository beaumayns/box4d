use crate::collision;
use crate::constraints;
use crate::draw_state;
use crate::input;
use crate::joint;
use crate::na;
use crate::physics;
use crate::sprite_renderer;

use crate::ga::Wedge;

#[derive(Clone)]
pub enum GrabState {
    Not,
    Miss,
    Hit(hecs::Entity),
}

pub struct Actor {
    pub grab_state: GrabState,
    pub move_thrust: f32,
    pub look_torque: f32,
}

#[rustfmt::skip]
pub fn update_actor(
    constraints: &mut constraints::Constraints,
    world: &mut hecs::World,
    input_state: &input::InputState,
    actor_entity: hecs::Entity,
) {
    // Handle movement in its own function, due to borrowing/mutability issues
    update_movement(world, input_state, actor_entity);

    let current_grab_state = world.get::<&Actor>(actor_entity).unwrap().grab_state.clone();
    match (input_state.grab, current_grab_state) {
        (true, GrabState::Not) => {
            let (actor_position, forward) = world
                .get::<&physics::RigidBody>(actor_entity)
                .map(|x| (x.position, x.orientation.to_matrix().column(2).normalize()))
                .unwrap();
            if let Some((hit_entity, t)) = collision::cast_ray(actor_position, forward, world) {
                let world_space_hit = world.get::<&physics::RigidBody>(actor_entity).unwrap().get_transform() * (t * na::vec4(0.0, 0.0, 1.0, 0.0));
                let world_to_local = world.get::<&physics::RigidBody>(hit_entity).unwrap().get_transform().inverse();
                constraints.add_joint(actor_entity, hit_entity, joint::Joint::new(
                    t * na::vec4(0.0, 0.0, 1.0, 0.0),
                    world_to_local * world_space_hit,
                ));

                world.query_one_mut::<&mut sprite_renderer::Sprite>(actor_entity).map(|mut x| x.tint = na::vec4(0.0, 1.0, 0.0, 1.0)).ok();
                world.query_one_mut::<&mut draw_state::DrawState>(hit_entity).map(|mut x| x.hollow = true).ok();
                world.query_one_mut::<&mut Actor>(actor_entity).map(|mut x| { x.grab_state = GrabState::Hit(hit_entity) }).ok();
            } else {
                world.query_one_mut::<&mut sprite_renderer::Sprite>(actor_entity).map(|mut x| x.tint = na::vec4(1.0, 0.0, 0.0, 1.0)).ok();
                world.query_one_mut::<&mut Actor>(actor_entity).map(|mut x| x.grab_state = GrabState::Miss).ok();
            }
        }
        (false, grab_state) => {
            world.query_one_mut::<&mut sprite_renderer::Sprite>(actor_entity).map(|mut x| x.tint = na::vec4(0.0, 0.0, 0.0, 1.0)).ok();
            world.query_one_mut::<&mut Actor>(actor_entity).map(|mut x| x.grab_state = GrabState::Not).ok();
            if let GrabState::Hit(hit_entity) = grab_state {
                world.query_one_mut::<&mut draw_state::DrawState>(hit_entity).map(|mut x| x.hollow = false).ok();
                constraints.remove_joint(actor_entity, hit_entity);
            }
        }
        _ => {}
    }
}

fn update_movement(
    world: &mut hecs::World,
    input_state: &input::InputState,
    actor_entity: hecs::Entity,
) {
    let mut actor_query = world
        .entity(actor_entity)
        .unwrap()
        .query::<(&mut Actor, &mut physics::RigidBody)>();
    let (actor, body) = actor_query.get().unwrap();
    let orientation = body.orientation.to_matrix();

    let right = orientation.column(0).normalize();
    let up = orientation.column(1).normalize();
    let forward = orientation.column(2).normalize();
    let ana = orientation.column(3).normalize();

    if input_state.up {
        body.force += up * actor.move_thrust;
    }
    if input_state.down {
        body.force -= up * actor.move_thrust;
    }
    if input_state.right {
        body.force += right * actor.move_thrust;
    }
    if input_state.left {
        body.force -= right * actor.move_thrust;
    }
    if input_state.forward {
        body.force += forward * actor.move_thrust;
    }
    if input_state.back {
        body.force -= forward * actor.move_thrust;
    }
    if input_state.ana {
        body.force += ana * actor.move_thrust;
    }
    if input_state.kata {
        body.force -= ana * actor.move_thrust;
    }

    let (yaw_plane, pitch_plane, roll_plane) = match input_state.hyperlook {
        false => (forward.wedge(right), up.wedge(forward), right.wedge(up)),
        true => (forward.wedge(ana), up.wedge(ana), right.wedge(ana)),
    };
    body.torque += actor.look_torque * input_state.yaw * yaw_plane;
    body.torque += actor.look_torque * input_state.pitch * pitch_plane;
    if input_state.roll_left {
        body.torque += 3.0 * actor.look_torque * roll_plane;
    }
    if input_state.roll_right {
        body.torque += -3.0 * actor.look_torque * roll_plane;
    }
}
