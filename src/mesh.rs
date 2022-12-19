use crate::na;
use crate::na::vec4;

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
    pub vertices: std::vec::Vec<na::Vector4>,
    normals: std::vec::Vec<na::Vector4>,
    colors: std::vec::Vec<na::Vector4>,
    indices: std::vec::Vec<u32>,
    pub num_tetrahedra: usize,
}

impl Mesh4 {
    pub fn transformed(self, transform: &na::Affine4) -> Self {
        Self {
            vertices: self.vertices.iter().map(|x| *transform * *x).collect(),
            normals: self.normals.iter().map(|x| transform.linear * *x).collect(),
            colors: self.colors,
            indices: self.indices,
            num_tetrahedra: self.num_tetrahedra,
        }
    }

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
            vec4(1.0, 0.8, 0.1, 1.0),
            vec4(0.0, 0.2, 0.3, 1.0),
            vec4(0.5, 0.0, 0.1, 1.0),
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
            .flat_map(|x| itertools::repeat_n(x, 7))
            .collect()
    }
}
