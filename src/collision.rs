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

pub fn do_collision(world: &mut hecs::World) {
    for (_, draw_state) in world.query_mut::<&mut draw_state::DrawState>() {
        draw_state.outline = na::vec4(0.0, 0.0, 0.0, 0.0);
    }

    for pair in world
        .query::<(&Collider, &physics::RigidBody)>()
        .iter()
        .combinations(2)
    {
        let (a, (a_collider, a_body)) = pair[0];
        let (b, (b_collider, b_body)) = pair[1];

        if (a_body.position - b_body.position).norm() < (a_collider.radius + b_collider.radius) {
            if let Some(t) = mpr::collide(
                a_collider,
                &a_body.get_transform(),
                b_collider,
                &b_body.get_transform(),
            ) {
                let brightness = t.clamp(0.0, 1.0);
                world
                    .get_mut::<draw_state::DrawState>(a)
                    .map(|mut x| x.outline = na::vec4(0.0, brightness, 0.0, 1.0))
                    .ok();
                world
                    .get_mut::<draw_state::DrawState>(b)
                    .map(|mut x| x.outline = na::vec4(0.0, brightness, 0.0, 1.0))
                    .ok();
            }
        }
    }
}
