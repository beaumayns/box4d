use crate::ga;
use crate::na;

#[derive(Debug, Copy, Clone)]
pub struct RigidBody {
    pub position: na::Vector4,
    pub orientation: ga::Rotor4,

    pub mass: f32,

    pub velocity: na::Vector4,
    pub force: na::Vector4,
    pub linear_damping: f32,

    pub angular_velocity: ga::Bivector4,
    pub torque: ga::Bivector4,
    pub angular_damping: f32,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            position: na::Vector4::zeros(),
            orientation: ga::Rotor4::identity(),

            mass: 1.0,

            velocity: na::Vector4::zeros(),
            force: na::Vector4::zeros(),
            linear_damping: 1.0,

            angular_velocity: ga::Bivector4::zero(),
            torque: ga::Bivector4::zero(),
            angular_damping: 1.0,
        }
    }
}

pub fn apply_physics(dt: f32, world: &mut hecs::World) {
    for (_, body) in world.query_mut::<&mut RigidBody>() {
        body.velocity *= body.linear_damping;
        body.velocity += dt * body.force / body.mass;
        body.force = na::Vector4::zeros();

        body.angular_velocity *= body.angular_damping;
        body.angular_velocity += dt * body.torque / body.mass; // TODO inertia tensor
        body.torque = ga::Bivector4::zero();

        body.position += body.velocity * dt;
        body.orientation *= ga::Rotor4::from_bivector(body.angular_velocity * dt);
    }
}
