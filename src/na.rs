pub type Matrix4 = nalgebra::Matrix4<f32>;
pub type Vector4 = nalgebra::SVector<f32, 4>;
pub type Vector2 = nalgebra::SVector<f32, 2>;

pub fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vector4 {
    Vector4::new(x, y, z, w)
}

pub fn vec2(x: f32, y: f32) -> Vector2 {
    Vector2::new(x, y)
}
