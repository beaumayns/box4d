use crate::collision;
use crate::ga;
use crate::na;

use crate::ga::{Reject, Wedge};

const E: f32 = 0.000001;

#[derive(Default, Copy, Clone)]
struct Vertex {
    a_local: na::Vector4,
    b_local: na::Vector4,
    minkowski: na::Vector4,
}

struct Simplex {
    center: na::Vector4,
    portal: [Vertex; 4],
}

impl Simplex {
    fn define_portal<F>(minkowski_sum: F, center: na::Vector4) -> Option<Self>
    where
        F: Fn(na::Vector4) -> Vertex,
    {
        let mut portal: [Vertex; 4] = [Vertex::default(); 4];

        // Search from the center in the direction of the origin
        let mut direction = -center;

        for i in 0..4 {
            // Find the next portal point
            portal[i] = minkowski_sum(direction);
            // If the new point is not on the other side of the origin from where we started, that's a fail
            if portal[i].minkowski.dot(&direction) < 0.0 {
                return None;
            }

            // Find a new direction to search - search orthogonally from the simplex formed by the center + existinng portal points,
            // in the direction of the origin
            direction = match i {
                0 => (portal[0].minkowski - center).reject(-portal[0].minkowski),
                1 => (portal[0].minkowski - center)
                    .wedge(portal[1].minkowski - center)
                    .reject(-portal[1].minkowski),
                2 => (portal[0].minkowski - center)
                    .wedge((portal[1].minkowski - center).wedge(portal[2].minkowski - center))
                    .reject(-portal[2].minkowski),
                _ => direction,
            };
            // If this happens, it means the origin is contained in that simplex. We could handle this as a special case of a smaller
            // overall simplex, but it's easier to just keep building a full simplex.
            if direction.norm_squared() == 0.0 {
                // Average the portal points relative to the center
                let x: na::Vector4 = portal
                    .iter()
                    .take(i + 1)
                    .map(|x| x.minkowski - center)
                    .fold(na::Vector4::zeros(), |x, y| x + y)
                    / (i + 1) as f32;
                // Search in some random direction orthogonal to that. Not sure if this is entirely reliable...
                direction = na::Vector4::new(-x[1], x[0], -x[3], x[2]);
            }
        }

        Some(Self { center, portal })
    }

    fn portal_trivector(&self) -> ga::Trivector4 {
        (self.portal[1].minkowski - self.portal[0].minkowski).wedge(
            (self.portal[2].minkowski - self.portal[0].minkowski)
                .wedge(self.portal[3].minkowski - self.portal[0].minkowski),
        )
    }

    // Check which of the existing portal points should be replaced, and replace it
    fn refine(&mut self, new_vertex: Vertex) -> bool {
        for i in 0..4 {
            // Form a matrix from a basis including the new portal point and excluding one of the old ones
            let m = na::Matrix4::from_columns(&[
                new_vertex.minkowski - self.center,
                self.portal[(i + 1) % 4].minkowski - self.center,
                self.portal[(i + 2) % 4].minkowski - self.center,
                self.portal[(i + 3) % 4].minkowski - self.center,
            ]);
            // Use that matrix to find coordinates of the minkowski-space origin in that basis
            if let Some(m_inv) = m.try_inverse() {
                let bc = m_inv * -self.center;
                // If the coordinates are all positive, the origin is within this new tetrahedron,
                // and the new point can replace the old. Should also technically check that the
                // sum of coordinates is <1, but that should be guaranteed by construction of these
                // points (I think)
                if bc[0] >= 0.0 && bc[1] >= 0.0 && bc[2] >= 0.0 && bc[3] >= 0.0 {
                    self.portal[i] = new_vertex;
                    return true;
                }
            }
        }
        // This should not happen but sometimes does, probably due to imprecise numerical
        // shenanigans. Give up and I guess we'll try again next frame.
        false
    }
}

pub fn collide(
    a_collider: &collision::Collider,
    a_transform: &na::Affine4,
    b_collider: &collision::Collider,
    b_transform: &na::Affine4,
) -> Option<f32> {
    let a_transform_inverse = a_transform.inverse().linear;
    let b_transform_inverse = b_transform.inverse().linear;

    // Helper lambda to do the minkowski difference for these particular
    // objects and transforms
    let minkowski_sum = |direction: na::Vector4| {
        let a_vertex = a_collider.support(&(a_transform_inverse * direction));
        let b_vertex = b_collider.support(&(b_transform_inverse * -direction));
        Vertex {
            a_local: a_vertex,
            b_local: b_vertex,
            minkowski: (a_transform * a_vertex) - (b_transform * b_vertex),
        }
    };

    // Define the initial portal
    let mut simplex = Simplex::define_portal(
        minkowski_sum,
        (a_transform * na::Vector4::zeros()) - (b_transform * na::Vector4::zeros()),
    )?;

    // Refine the portal
    // At this point, the origin lies within the frustum defined by the center and
    // the points of the portal. It may not yet lie behind the plane of the portal -
    // replace portal points until we can accomplish this.
    for i in 0..10 {
        let portal_trivector = simplex.portal_trivector();
        let direction = portal_trivector.reject(simplex.portal[0].minkowski - simplex.center);

        // Origin is behind the plane - we did it
        if direction.dot(&simplex.portal[0].minkowski) >= 0.0 {
            break;
        }

        // Otherwise continue searching
        let new_portal_vertex = minkowski_sum(direction);
        // If the new point was not on the other side of the origin,
        // or it just didn't move very far from the portal,
        // or this was the last chance
        // give up
        if new_portal_vertex.minkowski.dot(&direction) < 0.0
            || portal_trivector
                .reject(new_portal_vertex.minkowski - simplex.portal[0].minkowski)
                .norm_squared()
                < E
            || i == 9
        {
            return None;
        }

        // If we fail to refine the simplex, something weird happened. Give up.
        if !simplex.refine(new_portal_vertex) {
            return None;
        }
    }

    // Optimize the portal
    // By this point, we know we have the origin in the volume defined by the center
    // and the portal. We might not have a minimal portal, though, so try to improve that
    for _ in 0..10 {
        let portal_trivector = simplex.portal_trivector();
        let direction = portal_trivector.reject(simplex.portal[0].minkowski - simplex.center);
        let new_portal_vertex = minkowski_sum(direction);

        // We didn't improve the portal much, time to quit
        if portal_trivector
            .reject(new_portal_vertex.minkowski - simplex.portal[0].minkowski)
            .norm_squared()
            < E
        {
            break;
        }

        // If we fail to refine the simplex, something weird happened. Current simplex
        // is probably good enough
        if !simplex.refine(new_portal_vertex) {
            break;
        }
    }

    Some(
        simplex
            .portal_trivector()
            .reject(-simplex.portal[0].minkowski)
            .norm(),
    )
}
