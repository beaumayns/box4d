#![allow(clippy::many_single_char_names)]

use crate::collision;
use crate::ga::{Reject, Wedge};
use crate::na;

const E: f32 = 0.00001;

#[derive(Copy, Clone, Debug)]
enum Simplex {
    Empty,
    Point(na::Vector4),
    Line(na::Vector4, na::Vector4),
    Triangle(na::Vector4, na::Vector4, na::Vector4),
    Tetrahedron(na::Vector4, na::Vector4, na::Vector4, na::Vector4),
    Fivecell(
        na::Vector4,
        na::Vector4,
        na::Vector4,
        na::Vector4,
        na::Vector4,
    ),
    Complete,
}

fn voronoi(simplex: Simplex) -> (Simplex, Option<na::Vector4>) {
    match simplex {
        Simplex::Point(a) => {
            if a.norm_squared() == 0.0 {
                (Simplex::Complete, None)
            } else {
                (simplex, Some(-a))
            }
        }
        Simplex::Line(b, a) => {
            if (b - a).dot(&-a) < 0.0 {
                voronoi(Simplex::Point(a))
            } else if (a - b).dot(&-b) < 0.0 {
                voronoi(Simplex::Point(b))
            } else {
                match (b - a).reject(-a) {
                    rejection if rejection.norm_squared() > 0.0 => (simplex, Some(rejection)),
                    _ => (Simplex::Complete, None),
                }
            }
        }
        Simplex::Triangle(c, b, a) => {
            if (b - a).reject(c - b).dot(&-a) < 0.0 {
                voronoi(Simplex::Line(b, a))
            } else if (c - a).reject(b - c).dot(&-a) < 0.0 {
                voronoi(Simplex::Line(c, a))
            } else if (b - c).reject(a - b).dot(&-b) < 0.0 {
                voronoi(Simplex::Line(b, c))
            } else {
                match (b - a).wedge(c - a).reject(-a) {
                    rejection if rejection.norm_squared() > 0.0 => (simplex, Some(rejection)),
                    _ => (Simplex::Complete, None),
                }
            }
        }
        Simplex::Tetrahedron(d, c, b, a) => {
            if (d - a).wedge(c - a).reject(b - c).dot(&-a) < 0.0 {
                voronoi(Simplex::Triangle(d, c, a))
            } else if (d - a).wedge(b - a).reject(c - b).dot(&-a) < 0.0 {
                voronoi(Simplex::Triangle(d, b, a))
            } else if (c - a).wedge(b - a).reject(d - b).dot(&-a) < 0.0 {
                voronoi(Simplex::Triangle(c, b, a))
            } else if (c - b).wedge(d - b).reject(a - b).dot(&-b) < 0.0 {
                voronoi(Simplex::Triangle(b, c, d))
            } else {
                match (b - a).wedge((c - a).wedge(d - a)).reject(-a) {
                    rejection if rejection.norm_squared() > 0.0 => (simplex, Some(rejection)),
                    _ => (Simplex::Complete, None),
                }
            }
        }
        Simplex::Fivecell(e, d, c, b, a) => {
            if (c - a).wedge((d - a).wedge(e - a)).reject(b - c).dot(&-a) < 0.0 {
                voronoi(Simplex::Tetrahedron(c, d, e, a))
            } else if (b - a).wedge((d - a).wedge(e - a)).reject(c - b).dot(&-a) < 0.0 {
                voronoi(Simplex::Tetrahedron(b, d, e, a))
            } else if (b - a).wedge((c - a).wedge(e - a)).reject(d - b).dot(&-a) < 0.0 {
                voronoi(Simplex::Tetrahedron(b, c, e, a))
            } else if (b - a).wedge((c - a).wedge(d - a)).reject(e - b).dot(&-a) < 0.0 {
                voronoi(Simplex::Tetrahedron(b, c, d, a))
            } else if (c - b).wedge((d - b).wedge(e - b)).reject(a - b).dot(&-b) < 0.0 {
                voronoi(Simplex::Tetrahedron(b, c, d, e))
            } else {
                (Simplex::Complete, None)
            }
        }
        _ => (simplex, None),
    }
}

fn add_point(simplex: Simplex, point: na::Vector4) -> Simplex {
    match simplex {
        Simplex::Empty => Simplex::Point(point),
        Simplex::Point(a) => Simplex::Line(a, point),
        Simplex::Line(a, b) => Simplex::Triangle(a, b, point),
        Simplex::Triangle(a, b, c) => Simplex::Tetrahedron(a, b, c, point),
        Simplex::Tetrahedron(a, b, c, d) => Simplex::Fivecell(a, b, c, d, point),
        _ => simplex,
    }
}

fn expand(simplex: Simplex, point: na::Vector4) -> Simplex {
    let rejection = match simplex {
        Simplex::Empty => na::Vector4::repeat(1.0),
        Simplex::Point(a) => point - a,
        Simplex::Line(a, b) => (b - a).reject(point - a),
        Simplex::Triangle(a, b, c) => (b - a).wedge(c - a).reject(point - a),
        Simplex::Tetrahedron(a, b, c, d) => (b - a).wedge((c - a).wedge(d - a)).reject(point - a),
        _ => na::Vector4::zeros(),
    };

    if rejection.norm_squared() > 0.0 {
        add_point(simplex, point)
    } else {
        simplex
    }
}

pub fn cast_ray(
    ray_origin: &na::Vector4,
    direction: &na::Vector4,
    collider: &collision::Collider,
    transform: &na::Affine4,
) -> Option<f32> {
    let mut simplex = Simplex::Empty;

    let inverse_transform = transform.inverse().linear;

    let mut t: f32 = 0.0;
    let mut origin = *ray_origin;
    let mut normal = origin - transform * na::Vector4::zeros();

    for _ in 0..32 {
        if normal.norm_squared() < E {
            return Some(t);
        }

        let point = transform * collider.support(&(inverse_transform * normal.normalize()));
        let w = origin - point;

        if normal.dot(&w) > 0.0 {
            if normal.dot(direction) > -E {
                return None;
            }

            let distance = normal.dot(&w) / normal.dot(direction);
            let shift = direction * distance;

            t -= distance;
            origin = ray_origin + t * direction;
            simplex = match simplex {
                Simplex::Point(a) => Simplex::Point(shift + a),
                Simplex::Line(a, b) => Simplex::Line(shift + a, shift + b),
                Simplex::Triangle(a, b, c) => Simplex::Triangle(shift + a, shift + b, shift + c),
                Simplex::Tetrahedron(a, b, c, d) => {
                    Simplex::Tetrahedron(shift + a, shift + b, shift + c, shift + d)
                }
                Simplex::Fivecell(a, b, c, d, e) => {
                    Simplex::Fivecell(shift + a, shift + b, shift + c, shift + d, shift + e)
                }
                _ => simplex,
            };
        }

        simplex = expand(simplex, point - origin);

        match voronoi(simplex) {
            (Simplex::Complete, _) => return Some(t),
            (new_simplex, Some(new_normal)) => {
                simplex = new_simplex;
                normal = new_normal;
            }
            _ => {}
        }
    }
    None
}
