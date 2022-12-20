use crate::ga;
use crate::na;
use crate::physics;

#[derive(Debug)]
pub struct Joint {
    pub a: hecs::Entity,
    pub a_anchor: na::Vector4,
    pub a_jacobian: na::Matrix4x6,

    pub b: hecs::Entity,
    pub b_anchor: na::Vector4,
    pub b_jacobian: na::Matrix4x6,

    pub effective_mass: na::Matrix4,
    pub bias: na::Vector4,
    pub impulse: na::Vector4,
}

impl Joint {
    pub fn new(
        a: hecs::Entity,
        a_anchor: na::Vector4,
        b: hecs::Entity,
        b_anchor: na::Vector4,
    ) -> Self {
        Self {
            a,
            a_anchor,
            a_jacobian: na::Matrix4x6::zeros(),

            b,
            b_anchor,
            b_jacobian: na::Matrix4x6::zeros(),

            effective_mass: na::Matrix4::zeros(),
            bias: na::Vector4::zeros(),
            impulse: na::Vector4::zeros(),
        }
    }

    pub fn prepare(
        &mut self,
        dt: f32,
        a_body: &mut physics::RigidBody,
        b_body: &mut physics::RigidBody,
    ) {
        let a_world_orientation_anchor = a_body.orientation.to_matrix() * self.a_anchor;
        let b_world_orientation_anchor = b_body.orientation.to_matrix() * self.b_anchor;

        self.a_jacobian = ga::Bivector4::dot_vector_matrix(a_world_orientation_anchor);
        self.b_jacobian = ga::Bivector4::dot_vector_matrix(b_world_orientation_anchor);

        // This is the main reason to convert bivectors to/from coefficient vectors -
        // I don't knoww another way to incorporate the inverse inertia tensor into
        // this calculation. We're just using a single float for now, but it should be
        // a 6x6 matrix representing the moments in each combination of the 6
        // basis bivectors.
        // It would be nice to deal with this entirely via GA, but I don't know how,
        // and this seems to work.
        self.effective_mass = (na::Matrix4::identity() * a_body.inverse_mass
            + self.a_jacobian * a_body.inverse_inertia_tensor * self.a_jacobian.transpose()
            + na::Matrix4::identity() * b_body.inverse_mass
            + self.b_jacobian * b_body.inverse_inertia_tensor * self.b_jacobian.transpose())
        .try_inverse()
        .unwrap();

        let a_world_space_anchor = a_world_orientation_anchor + a_body.position;
        let b_world_space_anchor = b_world_orientation_anchor + b_body.position;
        self.bias = (a_world_space_anchor - b_world_space_anchor) * 0.5 / dt;

        // Warm starting - impulse is likely to be similar to last frame's
        a_body.velocity += a_body.inverse_mass * self.impulse;
        a_body.angular_velocity += ga::Bivector4::from_vector(
            a_body.inverse_inertia_tensor * self.a_jacobian.transpose() * self.impulse,
        );
        b_body.velocity -= b_body.inverse_mass * self.impulse;
        b_body.angular_velocity -= ga::Bivector4::from_vector(
            b_body.inverse_inertia_tensor * self.b_jacobian.transpose() * self.impulse,
        );
    }

    pub fn apply(&mut self, a_body: &mut physics::RigidBody, b_body: &mut physics::RigidBody) {
        let velocity = (a_body.velocity + self.a_jacobian * a_body.angular_velocity.as_vector())
            - (b_body.velocity + self.b_jacobian * b_body.angular_velocity.as_vector());

        let d_impulse = self.effective_mass * -(velocity + self.bias);
        self.impulse += d_impulse;

        a_body.velocity += a_body.inverse_mass * d_impulse;
        a_body.angular_velocity += ga::Bivector4::from_vector(
            a_body.inverse_inertia_tensor * self.a_jacobian.transpose() * d_impulse,
        );
        b_body.velocity -= b_body.inverse_mass * d_impulse;
        b_body.angular_velocity -= ga::Bivector4::from_vector(
            b_body.inverse_inertia_tensor * self.b_jacobian.transpose() * d_impulse,
        );
    }
}
