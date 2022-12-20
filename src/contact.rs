use crate::ga;
use crate::na;
use crate::physics;

use crate::ga::Wedge;

#[derive(Debug, Copy, Clone, Default)]
pub struct ContactPoint {
    pub a_local: na::Vector4,
    pub b_local: na::Vector4,
    pub normal: na::Vector4,
}

pub struct Contact {
    valid: bool,

    a_local: na::Vector4,
    a_world_space: na::Vector4,
    a_world_space_anchor: na::Vector4,
    a_world_space_original: na::Vector4,

    b_local: na::Vector4,
    b_world_space: na::Vector4,
    b_world_space_anchor: na::Vector4,
    b_world_space_original: na::Vector4,

    depth: f32,
    bias: f32,
    basis: na::Matrix4,

    impulse: na::Vector4,
    effective_masses: na::Vector4,
}

impl Contact {
    pub fn from_contact_point(
        contact_point: ContactPoint,
        a_body: &physics::RigidBody,
        b_body: &physics::RigidBody,
    ) -> Self {
        let normal = contact_point.normal.normalize();
        let basis = na::Matrix4::from_columns(&[
            normal,
            na::Vector4::new(-normal[1], normal[0], -normal[3], normal[2]),
            na::Vector4::new(normal[2], -normal[3], -normal[0], normal[1]),
            na::Vector4::new(normal[3], normal[2], -normal[1], -normal[0]),
        ]);
        let mut new = Self {
            valid: true,

            a_local: contact_point.a_local,
            a_world_space: na::Vector4::repeat(0.0),
            a_world_space_anchor: na::Vector4::repeat(0.0),
            a_world_space_original: na::Vector4::repeat(0.0),

            b_local: contact_point.b_local,
            b_world_space: na::Vector4::repeat(0.0),
            b_world_space_anchor: na::Vector4::repeat(0.0),
            b_world_space_original: na::Vector4::repeat(0.0),

            depth: contact_point.normal.norm(),
            bias: 0.0,
            basis,

            impulse: na::Vector4::default(),
            effective_masses: na::Vector4::default(),
        };
        new.update(true, a_body, b_body);
        new
    }

    pub fn update(
        &mut self,
        first: bool,
        a_body: &physics::RigidBody,
        b_body: &physics::RigidBody,
    ) {
        self.a_world_space_anchor = a_body.orientation.to_matrix() * self.a_local;
        self.a_world_space = a_body.position + self.a_world_space_anchor;
        self.b_world_space_anchor = b_body.orientation.to_matrix() * self.b_local;
        self.b_world_space = b_body.position + self.b_world_space_anchor;

        let separation = self.b_world_space - self.a_world_space;
        if first {
            self.a_world_space_original = self.a_world_space;
            self.b_world_space_original = self.b_world_space;
        } else if separation.dot(&self.basis.column(0)) > 0.0
            && ((self.a_world_space - self.a_world_space_original).transpose()
                * self.basis.fixed_slice::<4, 3>(0, 1))
            .norm()
                < 0.01
            && ((self.b_world_space - self.b_world_space_original).transpose()
                * self.basis.fixed_slice::<4, 3>(0, 1))
            .norm()
                < 0.01
        {
            self.depth = separation.dot(&self.basis.column(0));
        } else {
            self.valid = false;
        }
    }

    pub fn prepare(
        &mut self,
        dt: f32,
        a_body: &mut physics::RigidBody,
        b_body: &mut physics::RigidBody,
    ) {
        const RESTITUTION: f32 = 0.1;
        const BIAS_FACTOR: f32 = 0.1;
        const DEPTH_SLOP: f32 = 0.001;
        const REBOUND_SLOP: f32 = 0.1;

        let a_jacobian =
            self.basis.transpose() * ga::Bivector4::dot_vector_matrix(self.a_world_space_anchor);
        let b_jacobian =
            self.basis.transpose() * ga::Bivector4::dot_vector_matrix(self.b_world_space_anchor);

        self.effective_masses = na::Vector4::repeat(a_body.inverse_mass + b_body.inverse_mass);
        for i in 0..4 {
            self.effective_masses[i] += (a_jacobian.row(i)
                * a_body.inverse_inertia_tensor
                * a_jacobian.row(i).transpose())[0];
            self.effective_masses[i] += (b_jacobian.row(i)
                * b_body.inverse_inertia_tensor
                * b_jacobian.row(i).transpose())[0];
        }
        self.effective_masses = na::Vector4::repeat(1.0).component_div(&self.effective_masses);

        let velocity = (a_body.velocity - b_body.velocity)
            + (a_body.angular_velocity.dot(&self.a_world_space_anchor)
                - b_body.angular_velocity.dot(&self.b_world_space_anchor));
        self.bias = ((BIAS_FACTOR * (self.depth - DEPTH_SLOP).max(0.0)) / dt)
            + RESTITUTION * (self.basis.column(0).dot(&velocity) - REBOUND_SLOP).max(0.0);

        let impulse_world = self.basis * self.impulse;
        a_body.velocity += impulse_world * a_body.inverse_mass;
        a_body.angular_velocity +=
            a_body.inverse_inertia_tensor * self.a_world_space_anchor.wedge(impulse_world);
        b_body.velocity -= impulse_world * b_body.inverse_mass;
        b_body.angular_velocity -=
            b_body.inverse_inertia_tensor * self.b_world_space_anchor.wedge(impulse_world);
    }

    pub fn apply(&mut self, a_body: &mut physics::RigidBody, b_body: &mut physics::RigidBody) {
        let velocity = a_body.velocity - b_body.velocity
            + a_body.angular_velocity.dot(&self.a_world_space_anchor)
            - b_body.angular_velocity.dot(&self.b_world_space_anchor);

        // impulses in each axis of the basis, based on each axis's effective mass
        let mut delta_impulse = self.effective_masses.component_mul(
            &((self.basis.transpose() * -velocity) + na::Vector4::new(self.bias, 0.0, 0.0, 0.0)),
        );

        // save original impulse so we can calculate actual delta afterwards
        let impulse_original = self.impulse;
        // cap or clamp each axis as appropriate
        self.impulse[0] = (delta_impulse[0] + self.impulse[0]).max(0.0);
        let max_tangential_impulse = 0.3 * self.impulse[0];
        for i in 1..4 {
            self.impulse[i] = (delta_impulse[i] + self.impulse[i])
                .clamp(-max_tangential_impulse, max_tangential_impulse);
        }
        // calculate the delta after capping/clamping
        delta_impulse = self.basis * (self.impulse - impulse_original);

        // Apply impulses
        a_body.velocity += delta_impulse * a_body.inverse_mass;
        a_body.angular_velocity +=
            a_body.inverse_inertia_tensor * self.a_world_space_anchor.wedge(delta_impulse);
        b_body.velocity -= delta_impulse * b_body.inverse_mass;
        b_body.angular_velocity -=
            b_body.inverse_inertia_tensor * self.b_world_space_anchor.wedge(delta_impulse);
    }
}

pub struct Arbiter {
    pub contacts: std::vec::Vec<Contact>,
}

impl Arbiter {
    pub fn new(
        contact_point: ContactPoint,
        a_body: &physics::RigidBody,
        b_body: &physics::RigidBody,
    ) -> Self {
        let contact = Contact::from_contact_point(contact_point, a_body, b_body);
        Self {
            contacts: vec![contact],
        }
    }

    // This whole algorithm seems sketchy as hell, but it works.
    pub fn update(
        &mut self,
        contact_point: ContactPoint,
        a_body: &physics::RigidBody,
        b_body: &physics::RigidBody,
    ) {
        let new_contact = Contact::from_contact_point(contact_point, a_body, b_body);

        // Update each contact with the new body positions. Those which have moved
        // too far or become non-contacting will be marked invalid.
        // Also determine which of the existing contacts is closest to the new contact
        let mut new_distance = f32::INFINITY;
        for contact in self.contacts.iter_mut() {
            contact.update(false, a_body, b_body);
            if contact.valid {
                new_distance = new_distance
                    .min((contact.a_world_space - new_contact.a_world_space).norm_squared());
                new_distance = new_distance
                    .min((contact.b_world_space - new_contact.b_world_space).norm_squared());
            }
        }
        self.contacts.retain(|x| x.valid);

        // Only keep the new contact if it's not too close to an existing one
        if new_distance > 0.0001 {
            self.contacts.push(new_contact);
        }

        if self.contacts.len() < 6 {
            return;
        }
        // If we've reached max contacts, we need to prune one of them. Find the
        // one that's closest to any of the others (roughly, this probably does not
        // achieve exactly that), and remove it.
        let mut nearest_distance = f32::INFINITY;
        let mut nearest: usize = 0;
        for i in 0..self.contacts.len() {
            for j in i + 1..self.contacts.len() {
                let mut distance = (self.contacts[i].a_world_space
                    - self.contacts[j].a_world_space)
                    .norm_squared();
                distance = distance.min(
                    (self.contacts[i].b_world_space - self.contacts[j].b_world_space)
                        .norm_squared(),
                );
                if distance < nearest_distance {
                    nearest_distance = distance;
                    if self.contacts[i].depth > self.contacts[j].depth {
                        nearest = j;
                    } else {
                        nearest = i;
                    }
                }
            }
        }
        self.contacts.remove(nearest);
    }
}
