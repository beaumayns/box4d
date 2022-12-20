use crate::ga;
use crate::na;

#[derive(Debug, Copy, Clone)]
pub struct RigidBody {
    pub position: na::Vector4,
    pub velocity: na::Vector4,
    pub orientation: ga::Rotor4,
    pub angular_velocity: ga::Bivector4,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            position: na::Vector4::zeros(),
            velocity: na::Vector4::zeros(),
            orientation: ga::Rotor4::identity(),
            angular_velocity: ga::Bivector4::zero(),
        }
    }
}

impl RigidBody {
    pub fn update(&mut self, dt: f32) {
        self.velocity += dt * -self.position * self.position.norm_squared();
        self.position += self.velocity * dt;
        self.orientation *= ga::Rotor4::from_bivector(self.angular_velocity * dt);
    }
}
