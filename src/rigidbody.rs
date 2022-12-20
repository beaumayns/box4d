use crate::ga;
use crate::na;

#[derive(Debug, Copy, Clone)]
pub struct RigidBody {
    pub mass: f32,

    pub position: na::Vector4,
    pub velocity: na::Vector4,
    pub force: na::Vector4,
    pub linear_damping: f32,

    pub orientation: ga::Rotor4,
    pub angular_velocity: ga::Bivector4,
    pub torque: ga::Bivector4,
    pub angular_damping: f32,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            mass: 1.0,

            position: na::Vector4::zeros(),
            velocity: na::Vector4::zeros(),
            force: na::Vector4::zeros(),
            linear_damping: 1.0,

            orientation: ga::Rotor4::identity(),
            angular_velocity: ga::Bivector4::zero(),
            torque: ga::Bivector4::zero(),
            angular_damping: 1.0,
        }
    }
}

impl RigidBody {
    pub fn update(&mut self, dt: f32) {
        self.velocity *= self.linear_damping;
        self.velocity += dt * self.force / self.mass;
        self.force = na::Vector4::zeros();

        self.angular_velocity *= self.angular_damping;
        self.angular_velocity += dt * self.torque / self.mass; // TODO inertia tensor
        self.torque = ga::Bivector4::zero();

        self.position += self.velocity * dt;
        self.orientation *= ga::Rotor4::from_bivector(self.angular_velocity * dt);
    }
}
