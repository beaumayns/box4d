use crate::constraints;
use crate::ga;
use crate::na;

#[derive(Debug, Copy, Clone)]
pub struct RigidBody {
    pub position: na::Vector4,
    pub orientation: ga::Rotor4,

    pub mass: f32,
    pub inverse_mass: f32,
    pub inertia_tensor: f32,
    pub inverse_inertia_tensor: f32,

    pub gravity: f32,

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
            inverse_mass: 1.0,

            // Just use a single number as the inertia tensor for now - it's close
            // enough. In reality, this should be a 6x6 symmetric matrix representing
            // the moments of the object relative to each of the basis bivectors.
            inertia_tensor: 1.0,
            inverse_inertia_tensor: 1.0,

            gravity: 1.0,

            velocity: na::Vector4::zeros(),
            force: na::Vector4::zeros(),
            linear_damping: 1.0,

            angular_velocity: ga::Bivector4::zero(),
            torque: ga::Bivector4::zero(),
            angular_damping: 1.0,
        }
    }
}

impl RigidBody {
    pub fn with_mass(self, mass: f32) -> Self {
        Self {
            mass,
            inverse_mass: 1.0 / mass,
            inertia_tensor: mass,
            inverse_inertia_tensor: 1.0 / mass,
            ..self
        }
    }

    pub fn get_transform(&self) -> na::Affine4 {
        na::Affine4::from_po(self.position, self.orientation.to_matrix())
    }
}

pub fn apply_physics(dt: f32, constraints: &mut constraints::Constraints, world: &mut hecs::World) {
    const GRAVITY: na::Vector4 = na::Vector4::new(0.0, -10.0, 0.0, 0.0);

    for (_, body) in world.query_mut::<&mut RigidBody>() {
        body.velocity *= body.linear_damping;
        body.velocity += dt * (GRAVITY * body.gravity + body.force / body.mass);
        body.force = na::Vector4::zeros();

        body.angular_velocity *= body.angular_damping;
        body.angular_velocity += dt * body.torque / body.mass; // TODO inertia tensor
        body.torque = ga::Bivector4::zero();
    }

    constraints.prepare(dt, world);
    for _ in 0..4 {
        constraints.apply(world);
    }

    for (_, body) in world.query_mut::<&mut RigidBody>() {
        body.position += body.velocity * dt;
        body.orientation *= ga::Rotor4::from_bivector(body.angular_velocity * dt);
    }
}
