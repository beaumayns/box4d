use crate::na;
use crate::na::vec4;

use itertools::Itertools;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Tetrahedron {
    positions: na::Matrix4,
    normals: na::Matrix4,
    colors: na::Matrix4,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Mix {
    a: u32,
    b: u32,
    p1: u32, // padding, since array elements in std140 layout are 16-btye aligned
    p2: u32, // padding
}

#[derive(Debug, Clone)]
pub struct Mesh4 {
    vertices: std::vec::Vec<na::Vector4>,
    normals: std::vec::Vec<na::Vector4>,
    colors: std::vec::Vec<na::Vector4>,
    indices: std::vec::Vec<u32>,
    pub num_tetrahedra: usize,
}

impl Mesh4 {
    pub fn cube() -> Self {
        #[rustfmt::skip]
        let vertices: Vec<na::Vector4> = vec![
            vec4( 0.5, -0.5, -0.5, -0.5), vec4( 0.5, -0.5, -0.5,  0.5), vec4( 0.5, -0.5,  0.5, -0.5), vec4( 0.5, -0.5,  0.5,  0.5), // + x
            vec4( 0.5,  0.5, -0.5, -0.5), vec4( 0.5,  0.5, -0.5,  0.5), vec4( 0.5,  0.5,  0.5, -0.5), vec4( 0.5,  0.5,  0.5,  0.5), // + x
            vec4(-0.5, -0.5, -0.5, -0.5), vec4(-0.5, -0.5, -0.5,  0.5), vec4(-0.5, -0.5,  0.5, -0.5), vec4(-0.5, -0.5,  0.5,  0.5), // - x
            vec4(-0.5,  0.5, -0.5, -0.5), vec4(-0.5,  0.5, -0.5,  0.5), vec4(-0.5,  0.5,  0.5, -0.5), vec4(-0.5,  0.5,  0.5,  0.5), // - x
            vec4(-0.5,  0.5, -0.5, -0.5), vec4(-0.5,  0.5, -0.5,  0.5), vec4(-0.5,  0.5,  0.5, -0.5), vec4(-0.5,  0.5,  0.5,  0.5), // + y
            vec4( 0.5,  0.5, -0.5, -0.5), vec4( 0.5,  0.5, -0.5,  0.5), vec4( 0.5,  0.5,  0.5, -0.5), vec4( 0.5,  0.5,  0.5,  0.5), // + y
            vec4(-0.5, -0.5, -0.5, -0.5), vec4(-0.5, -0.5, -0.5,  0.5), vec4(-0.5, -0.5,  0.5, -0.5), vec4(-0.5, -0.5,  0.5,  0.5), // - y
            vec4( 0.5, -0.5, -0.5, -0.5), vec4( 0.5, -0.5, -0.5,  0.5), vec4( 0.5, -0.5,  0.5, -0.5), vec4( 0.5, -0.5,  0.5,  0.5), // - y
            vec4(-0.5, -0.5,  0.5, -0.5), vec4(-0.5, -0.5,  0.5,  0.5), vec4(-0.5,  0.5,  0.5, -0.5), vec4(-0.5,  0.5,  0.5,  0.5), // + z
            vec4( 0.5, -0.5,  0.5, -0.5), vec4( 0.5, -0.5,  0.5,  0.5), vec4( 0.5,  0.5,  0.5, -0.5), vec4( 0.5,  0.5,  0.5,  0.5), // + z
            vec4(-0.5, -0.5, -0.5, -0.5), vec4(-0.5, -0.5, -0.5,  0.5), vec4(-0.5,  0.5, -0.5, -0.5), vec4(-0.5,  0.5, -0.5,  0.5), // - z
            vec4( 0.5, -0.5, -0.5, -0.5), vec4( 0.5, -0.5, -0.5,  0.5), vec4( 0.5,  0.5, -0.5, -0.5), vec4( 0.5,  0.5, -0.5,  0.5), // - z
            vec4(-0.5, -0.5, -0.5,  0.5), vec4(-0.5, -0.5,  0.5,  0.5), vec4(-0.5,  0.5, -0.5,  0.5), vec4(-0.5,  0.5,  0.5,  0.5), // + w
            vec4( 0.5, -0.5, -0.5,  0.5), vec4( 0.5, -0.5,  0.5,  0.5), vec4( 0.5,  0.5, -0.5,  0.5), vec4( 0.5,  0.5,  0.5,  0.5), // + w
            vec4(-0.5, -0.5, -0.5, -0.5), vec4(-0.5, -0.5,  0.5, -0.5), vec4(-0.5,  0.5, -0.5, -0.5), vec4(-0.5,  0.5,  0.5, -0.5), // - w
            vec4( 0.5, -0.5, -0.5, -0.5), vec4( 0.5, -0.5,  0.5, -0.5), vec4( 0.5,  0.5, -0.5, -0.5), vec4( 0.5,  0.5,  0.5, -0.5), // - w
        ];
        let normals: Vec<na::Vector4> = [
            vec4(1.0, 0.0, 0.0, 0.0),
            vec4(-1.0, 0.0, 0.0, 0.0),
            vec4(0.0, 1.0, 0.0, 0.0),
            vec4(0.0, -1.0, 0.0, 0.0),
            vec4(0.0, 0.0, 1.0, 0.0),
            vec4(0.0, 0.0, -1.0, 0.0),
            vec4(0.0, 0.0, 0.0, 1.0),
            vec4(0.0, 0.0, 0.0, -1.0),
        ]
        .iter()
        .flat_map(|x| [*x].repeat(8))
        .collect();

        let colors: Vec<na::Vector4> = [
            vec4(0.0, 0.2, 0.3, 1.0),
            vec4(0.5, 0.0, 0.1, 1.0),
            vec4(1.0, 0.8, 0.0, 1.0),
            vec4(0.3, 0.0, 0.4, 1.0),
        ]
        .iter()
        .flat_map(|x| [*x].repeat(16))
        .collect();

        let mut indices: Vec<u32> = Vec::new();
        for i in 0..(vertices.len() / 8) as u32 {
            let ix = i * 8;

            indices.extend_from_slice(&[ix, ix + 1, ix + 2, ix + 4]);
            indices.extend_from_slice(&[ix + 1, ix + 6, ix + 5, ix + 4]);
            indices.extend_from_slice(&[ix + 1, ix + 2, ix + 4, ix + 6]);

            indices.extend_from_slice(&[ix + 1, ix + 3, ix + 6, ix + 5]);
            indices.extend_from_slice(&[ix + 3, ix + 5, ix + 7, ix + 6]);
            indices.extend_from_slice(&[ix + 3, ix + 2, ix + 1, ix + 6]);
        }

        let num_tetrahedra = indices.len() / 4;

        Self {
            vertices,
            normals,
            colors,
            indices,
            num_tetrahedra,
        }
    }

    pub fn get_buffer_data(&self) -> Vec<Tetrahedron> {
        (0..self.indices.len())
            .step_by(4)
            .map(|i| Tetrahedron {
                positions: na::Matrix4::from_columns(&[
                    self.vertices[self.indices[i] as usize],
                    self.vertices[self.indices[i + 1] as usize],
                    self.vertices[self.indices[i + 2] as usize],
                    self.vertices[self.indices[i + 3] as usize],
                ]),
                normals: na::Matrix4::from_columns(&[
                    self.normals[self.indices[i] as usize],
                    self.normals[self.indices[i + 1] as usize],
                    self.normals[self.indices[i + 2] as usize],
                    self.normals[self.indices[i + 3] as usize],
                ]),
                colors: na::Matrix4::from_columns(&[
                    self.colors[self.indices[i] as usize],
                    self.colors[self.indices[i + 1] as usize],
                    self.colors[self.indices[i + 2] as usize],
                    self.colors[self.indices[i + 3] as usize],
                ]),
            })
            .collect()
    }
}

pub fn mixtable_bytes() -> Vec<u8> {
    // A tetrahedron has 4 vertices, and each one of those vertices may be "above",
    // in, or "below" the view-hyperplane. This gives 81 possible situations, resulting
    // in either nothing (or a mere point or line), a triangle, a quad, or the entire
    // tetrahedron being rendered.
    // Each tetrahedron is sent to the vertex shader 7 times in order ot create a
    // triangle strip at least long enough to cover rending the full tetrahedron (6
    // vertices on the strip), plus a leading degenerate triangle to avoid weird
    // inter-tetrahedral triangles.
    // Each of the 7 invocations corresponds results in a new vertex on the strip
    // being created from some combination of the vertices in the tetrahedron, based
    // on which of 81 possible strips is relevant to the tetrahedron.
    [[0, 1, 2]]
        .repeat(4)
        .into_iter()
        .multi_cartesian_product()
        .map(|signs| {
        // sort each vertex into above, below, or in plane
        let mut inplane: Vec<u32> = Vec::new();
        let mut above: Vec<u32> = Vec::new();
        let mut below: Vec<u32> = Vec::new();
        for (j, s) in signs.iter().enumerate() {
            match s {
                0 => below.push(j as u32),
                1 => inplane.push(j as u32),
                2 => above.push(j as u32),
                _ => (),
            }
        }

        // create a mix for each vertex or combination of vertices that intersects the plane
        let mut combinations: Vec<Mix> = Vec::new();
        for vi in &inplane {
            combinations.push(Mix {
                a: *vi,
                b: *vi,
                p1: 0,
                p2: 0,
            });
        }
        for avi in above {
            for bvi in &below {
                combinations.push(Mix {
                    a: avi,
                    b: *bvi,
                    p1: 0,
                    p2: 0,
                });
            }
        }

        let mut row: [Mix; 7] = [Mix::default(); 7];
        // If more than 3 point in the plane, set up the combinations that will
        // make up the vertices of the triangles formed by those points
        if combinations.len() >= 3 {
            // The first triangle is always the same - it includes a duplicate
            // point in the beginning, to bridge it to the previous tetrahedron
            // in the triangle strip.
            row[0] = combinations[0];
            row[1] = combinations[0];
            row[2] = combinations[1];
            row[3] = combinations[2];

            if combinations.len() == 3 {
                // If there are exactly 3 points in the plane, fill the rest of
                // of the strip with duplicates to make degenerate triangles
                row[4] = combinations[2];
                row[5] = combinations[2];
                row[6] = combinations[2];
            } else if combinations.len() == 4 {
                // Otherwise, the result is at least a quad - add another
                // meaningful point
                row[4] = combinations[3];
                if inplane.len() == 4 {
                    // If all vertices were exactly in the plane, the result
                    // should be the full tetrahedron - fill out the triangle
                    // strip with the rest of its vertices
                    row[5] = combinations[0];
                    row[6] = combinations[1];
                } else {
                    // Otherwise, it was just a quad after all - fill with
                    // duplicates to induce degeneracy
                    row[5] = combinations[3];
                    row[6] = combinations[3];
                }
            }
        }
        row
    })
    .flat_map(|x| bytemuck::bytes_of(&x).to_vec())
    .collect::<Vec<u8>>()
}
