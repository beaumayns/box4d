pub type Matrix4 = nalgebra::Matrix4<f32>;
pub type Vector4 = nalgebra::SVector<f32, 4>;

pub fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vector4 {
    Vector4::new(x, y, z, w)
}
