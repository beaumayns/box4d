use crate::input;
use crate::physics;

use crate::ga::Wedge;

pub struct Actor {
    pub move_thrust: f32,
    pub look_torque: f32,
}

pub fn update_actor(
    world: &mut hecs::World,
    input_state: &input::InputState,
    actor_entity: hecs::Entity,
) {
    let (actor, body) = world
        .query_one_mut::<(&Actor, &mut physics::RigidBody)>(actor_entity)
        .unwrap();
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
