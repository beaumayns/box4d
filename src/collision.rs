use crate::gjk;
use crate::mesh;
use crate::na;
use crate::physics;

use itertools::Itertools;
use std::cmp::Ordering::Greater;

#[derive(Debug, Clone)]
pub struct Collider {
    hull: Vec<na::Vector4>,
}

impl Collider {
    pub fn from_mesh4(mesh: &mesh::Mesh4) -> Self {
        Self {
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
