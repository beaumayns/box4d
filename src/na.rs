use std::ops::{Mul, MulAssign};

pub type Matrix4 = nalgebra::Matrix4<f32>;
pub type Matrix4x6 = nalgebra::SMatrix<f32, 4, 6>;
pub type Vector2 = nalgebra::SVector<f32, 2>;
pub type Vector4 = nalgebra::SVector<f32, 4>;
pub type Vector6 = nalgebra::SVector<f32, 6>;

pub fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vector4 {
    Vector4::new(x, y, z, w)
}

pub fn vec2(x: f32, y: f32) -> Vector2 {
    Vector2::new(x, y)
}

#[derive(Debug, Clone, Copy)]
pub struct Affine4 {
    pub linear: Matrix4,
    pub translation: Vector4,
}

impl Affine4 {
    pub fn identity() -> Self {
        Self {
            linear: Matrix4::identity(),
            translation: Vector4::zeros(),
        }
    }

    pub fn from_po(position: Vector4, orientation: Matrix4) -> Self {
        Self {
            linear: orientation,
            translation: position,
        }
    }

    pub fn inverse(&self) -> Self {
        // The linear part of this is (in this program at least) guaranteed to be
        // orthogonal, so the transpose is the inverse
        let linear_inverse = self.linear.transpose();
        Affine4 {
            linear: linear_inverse,
            translation: -(linear_inverse * self.translation),
        }
    }
}

impl Default for Affine4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul<Vector4> for Affine4 {
    type Output = Vector4;

    fn mul(self, rhs: Vector4) -> Vector4 {
        self.translation + self.linear * rhs
    }
}

impl Mul<Vector4> for &Affine4 {
    type Output = Vector4;

    fn mul(self, rhs: Vector4) -> Vector4 {
        self.translation + self.linear * rhs
    }
}

impl Mul<Affine4> for Affine4 {
    type Output = Affine4;

    fn mul(self, rhs: Affine4) -> Affine4 {
        let mut r = self;
        r *= rhs;
        r
    }
}

impl MulAssign<Affine4> for Affine4 {
    fn mul_assign(&mut self, rhs: Affine4) {
        self.translation += self.linear * rhs.translation;
        self.linear = self.linear * rhs.linear;
    }
}
