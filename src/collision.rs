use crate::constraints;
use crate::contact;
use crate::draw_state;
use crate::gjk;
use crate::mesh;
use crate::mpr;
use crate::na;
use crate::physics;

use itertools::Itertools;
use std::cmp::Ordering::Greater;

#[derive(Debug, Clone)]
pub struct Collider {
    radius: f32,
    hull: Vec<na::Vector4>,
}

impl Collider {
    pub fn from_mesh4(mesh: &mesh::Mesh4) -> Self {
        Self {
            radius: mesh
                .vertices
                .iter()
                .map(|x| x.norm())
                .fold(0.0, |x, y| x.max(y)),
            hull: mesh
                .vertices
                .iter()
                .unique_by(|v| bytemuck::bytes_of(*v))
                .cloned()
                .collect(),
        }
    }

    pub fn support(&self, direction: &na::Vector4) -> na::Vector4 {
        *self
            .hull
            .iter()
            .max_by(|x, y| {
                x.dot(direction)
                    .partial_cmp(&y.dot(direction))
                    .unwrap_or(Greater)
            })
            .unwrap()
    }
}

pub fn cast_ray(
    origin: na::Vector4,
    direction: na::Vector4,
    world: &hecs::World,
) -> Option<(hecs::Entity, f32)> {
    let mut nearest = None;
    let mut nearest_distance = f32::INFINITY;
    for (entity, (body, collider)) in world.query::<(&physics::RigidBody, &Collider)>().iter() {
        if let Some(t) = gjk::cast_ray(
            &origin,
            &direction,
            collider,
            &na::Affine4::from_po(body.position, body.orientation.to_matrix()),
        ) {
            if t < nearest_distance {
                nearest = Some(entity);
                nearest_distance = t;
            }
        }
    }
    nearest.map(|entity| (entity, nearest_distance))
}

pub fn do_collisions(constraints: &mut constraints::Constraints, world: &mut hecs::World) {
    let mut draw_state_query = world.query::<&mut draw_state::DrawState>();
    for pair in world
        .query::<(&Collider, &physics::RigidBody)>()
        .iter()
        .combinations(2)
    {
        let (a, (a_collider, a_body)) = pair[0];
        let (b, (b_collider, b_body)) = pair[1];

        let arbiter = constraints.get_arbiter(a, b);
        let contact = if (a_body.position - b_body.position).norm()
            < (a_collider.radius + b_collider.radius)
        {
            mpr::collide(
                a_collider,
                &a_body.get_transform(),
                b_collider,
                &b_body.get_transform(),
            )
        } else {
            None
        };

        match (arbiter, contact) {
            (None, None) => {}
            (None, Some(contact_point)) => {
                constraints.add_arbiter(a, b, contact::Arbiter::new(contact_point, a_body, b_body));
                if let Some(mut state) = draw_state_query.view().get_mut(a) {
                    state.contacts += 1;
                }
                if let Some(mut state) = draw_state_query.view().get_mut(b) {
                    state.contacts += 1;
                }
            }
            (Some(arbiter), Some(contact_point)) => {
                arbiter.update(contact_point, a_body, b_body);
            }
            (Some(_), None) => {
                constraints.remove_arbiter(a, b);
                draw_state_query
                    .view()
                    .get_mut(a)
                    .map_or((), |mut state| state.contacts -= 1);
                draw_state_query
                    .view()
                    .get_mut(b)
                    .map_or((), |mut state| state.contacts -= 1);
            }
        }
    }
}
