use crate::contact;
use crate::joint;
use crate::physics;

pub struct Constraints {
    joints: std::collections::HashMap<(hecs::Entity, hecs::Entity), joint::Joint>,
    arbiters: std::collections::HashMap<(hecs::Entity, hecs::Entity), contact::Arbiter>,
}

impl Constraints {
    pub fn new() -> Self {
        Self {
            joints: std::collections::HashMap::new(),
            arbiters: std::collections::HashMap::new(),
        }
    }

    pub fn add_joint(
        &mut self,
        a: hecs::Entity,
        b: hecs::Entity,
        joint: joint::Joint,
    ) -> Option<joint::Joint> {
        self.joints.insert((a, b), joint)
    }

    pub fn remove_joint(&mut self, a: hecs::Entity, b: hecs::Entity) -> Option<joint::Joint> {
        self.joints.remove(&(a, b))
    }

    pub fn add_arbiter(
        &mut self,
        a: hecs::Entity,
        b: hecs::Entity,
        arbiter: contact::Arbiter,
    ) -> Option<contact::Arbiter> {
        self.arbiters.insert((a, b), arbiter)
    }

    pub fn get_arbiter(
        &mut self,
        a: hecs::Entity,
        b: hecs::Entity,
    ) -> Option<&mut contact::Arbiter> {
        self.arbiters.get_mut(&(a, b))
    }

    pub fn remove_arbiter(&mut self, a: hecs::Entity, b: hecs::Entity) -> Option<contact::Arbiter> {
        self.arbiters.remove(&(a, b))
    }

    pub fn prepare(&mut self, dt: f32, world: &mut hecs::World) {
        let mut body_query = world.query_mut::<&mut physics::RigidBody>();
        let mut body_view = body_query.view();
        for ((a, b), joint) in self.joints.iter_mut() {
            let [a_body, b_body] = body_view.get_mut_n([*a, *b]).map(|x| x.unwrap());
            joint.prepare(dt, a_body, b_body);
        }
        for ((a, b), arbiter) in self.arbiters.iter_mut() {
            let [a_body, b_body] = body_view.get_mut_n([*a, *b]).map(|x| x.unwrap());
            for contact in arbiter.contacts.iter_mut() {
                contact.prepare(dt, a_body, b_body);
            }
        }
    }

    pub fn apply(&mut self, world: &mut hecs::World) {
        let mut body_query = world.query_mut::<&mut physics::RigidBody>();
        let mut body_view = body_query.view();
        for ((a, b), joint) in self.joints.iter_mut() {
            let [a_body, b_body] = body_view.get_mut_n([*a, *b]).map(|x| x.unwrap());
            joint.apply(a_body, b_body);
        }
        for ((a, b), arbiter) in self.arbiters.iter_mut() {
            let [a_body, b_body] = body_view.get_mut_n([*a, *b]).map(|x| x.unwrap());
            for contact in arbiter.contacts.iter_mut() {
                contact.apply(a_body, b_body);
            }
        }
    }
}
