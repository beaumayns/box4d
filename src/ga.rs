use crate::na;
use std::ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy)]
pub struct Bivector4 {
    c: [f32; 6],
}

impl Bivector4 {
    pub fn zero() -> Self {
        Self {
            c: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn norm(&self) -> f32 {
        self.norm_squared().sqrt()
    }

    pub fn norm_squared(&self) -> f32 {
        self.c.iter().map(|&x| x * x).sum()
    }

    // If v represents the point on the surface of a rotating object, then this
    // function will return a matrix which, when used to multiply from the left
    // a vector representing the coefficients of a bivector, will calculate the dot
    // product of that bivector and v. The result of this will be a vector
    // representing the linear velocity at point v.
    // In other words, this is a jacobian matrix - it converts from world-space
    // rotational velocity to linear velocity at point v.
    #[rustfmt::skip]
    pub fn dot_vector_matrix(v: na::Vector4) -> na::Matrix4x6 {
        na::Matrix4x6::new(
            -v[1], -v[2], -v[3],   0.0,   0.0,   0.0,
             v[0],   0.0,   0.0, -v[2], -v[3],   0.0,
              0.0,  v[0],   0.0,  v[1],   0.0, -v[3],
              0.0,   0.0,  v[0],   0.0,  v[1],  v[2],
        )
    }

    pub fn from_vector(v: na::Vector6) -> Self {
        Self {
            c: v.as_slice().try_into().unwrap(),
        }
    }

    pub fn as_vector(&self) -> na::Vector6 {
        na::Vector6::from_column_slice(&self.c)
    }

    pub fn dot(&self, v: &na::Vector4) -> na::Vector4 {
        na::Vector4::new(
            -self[0] * v[1] - self[1] * v[2] - self[2] * v[3],
            self[0] * v[0] - self[3] * v[2] - self[4] * v[3],
            self[1] * v[0] + self[3] * v[1] - self[5] * v[3],
            self[2] * v[0] + self[4] * v[1] + self[5] * v[2],
        )
    }
}

impl Index<usize> for Bivector4 {
    type Output = f32;

    fn index(&self, i: usize) -> &f32 {
        &self.c[i]
    }
}

impl Add<Bivector4> for Bivector4 {
    type Output = Self;

    fn add(self, rhs: Bivector4) -> Self {
        let mut r = self;
        r += rhs;
        r
    }
}

impl AddAssign<Bivector4> for Bivector4 {
    fn add_assign(&mut self, rhs: Bivector4) {
        for (i, v) in &mut self.c.iter_mut().enumerate() {
            *v += rhs[i];
        }
    }
}

impl Sub<Bivector4> for Bivector4 {
    type Output = Self;

    fn sub(self, rhs: Bivector4) -> Self {
        let mut r = self;
        r -= rhs;
        r
    }
}

impl SubAssign<Bivector4> for Bivector4 {
    fn sub_assign(&mut self, rhs: Bivector4) {
        for (i, v) in &mut self.c.iter_mut().enumerate() {
            *v -= rhs[i];
        }
    }
}

impl Mul<Bivector4> for f32 {
    type Output = Bivector4;

    fn mul(self, rhs: Bivector4) -> Bivector4 {
        let mut r: Bivector4 = rhs;
        for x in &mut r.c {
            *x *= self;
        }
        r
    }
}

impl Mul<f32> for Bivector4 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        let mut r = self;
        r *= rhs;
        r
    }
}

impl MulAssign<f32> for Bivector4 {
    fn mul_assign(&mut self, rhs: f32) {
        for x in &mut self.c {
            *x *= rhs;
        }
    }
}

impl Div<f32> for Bivector4 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        let mut mc = self.c;
        for x in &mut mc {
            *x /= rhs;
        }
        Self { c: mc }
    }
}

impl DivAssign<f32> for Bivector4 {
    fn div_assign(&mut self, rhs: f32) {
        for x in &mut self.c {
            *x /= rhs;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rotor4 {
    c: [f32; 8],
}

impl Rotor4 {
    pub fn identity() -> Self {
        Self {
            c: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn from_bivector(bv: Bivector4) -> Rotor4 {
        let m = bv.norm();
        let mut c: [f32; 8] = [0.0; 8];

        c[0] = m.cos();
        if m != 0.0 {
            for i in 1..7 {
                c[i] = m.sin() * bv[i - 1] / m;
            }
        }

        Rotor4 { c }
    }

    pub fn to_matrix(self) -> na::Matrix4 {
        let [c0, c1, c2, c3, c4, c5, c6, c7] = self.c;

        na::Matrix4::from_row_slice(&[
            c0 * c0 - c1 * c1 - c2 * c2 - c3 * c3 + c4 * c4 + c5 * c5 + c6 * c6 - c7 * c7,
            -2.0 * c0 * c1 - 2.0 * c2 * c4 - 2.0 * c3 * c5 - 2.0 * c6 * c7,
            -2.0 * c0 * c2 + 2.0 * c1 * c4 - 2.0 * c3 * c6 + 2.0 * c5 * c7,
            -2.0 * c0 * c3 + 2.0 * c1 * c5 + 2.0 * c2 * c6 - 2.0 * c4 * c7,
            2.0 * c0 * c1 - 2.0 * c2 * c4 - 2.0 * c3 * c5 + 2.0 * c6 * c7,
            c0 * c0 - c1 * c1 + c2 * c2 + c3 * c3 - c4 * c4 - c5 * c5 + c6 * c6 - c7 * c7,
            -2.0 * c0 * c4 - 2.0 * c1 * c2 - 2.0 * c3 * c7 - 2.0 * c5 * c6,
            -2.0 * c0 * c5 - 2.0 * c1 * c3 + 2.0 * c2 * c7 + 2.0 * c4 * c6,
            2.0 * c0 * c2 + 2.0 * c1 * c4 - 2.0 * c3 * c6 - 2.0 * c5 * c7,
            2.0 * c0 * c4 - 2.0 * c1 * c2 + 2.0 * c3 * c7 - 2.0 * c5 * c6,
            c0 * c0 + c1 * c1 - c2 * c2 + c3 * c3 - c4 * c4 + c5 * c5 - c6 * c6 - c7 * c7,
            -2.0 * c0 * c6 - 2.0 * c1 * c7 - 2.0 * c2 * c3 - 2.0 * c4 * c5,
            2.0 * c0 * c3 + 2.0 * c1 * c5 + 2.0 * c2 * c6 + 2.0 * c4 * c7,
            2.0 * c0 * c5 - 2.0 * c1 * c3 - 2.0 * c2 * c7 + 2.0 * c4 * c6,
            2.0 * c0 * c6 + 2.0 * c1 * c7 - 2.0 * c2 * c3 - 2.0 * c4 * c5,
            c0 * c0 + c1 * c1 + c2 * c2 - c3 * c3 + c4 * c4 - c5 * c5 - c6 * c6 - c7 * c7,
        ])
    }
}

impl Mul for Rotor4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut r = self;
        r *= rhs;
        r
    }
}

impl MulAssign for Rotor4 {
    fn mul_assign(&mut self, rhs: Self) {
        let [l0, l1, l2, l3, l4, l5, l6, l7] = self.c;
        let [r0, r1, r2, r3, r4, r5, r6, r7] = rhs.c;

        self.c[0] = l0 * r0 - l1 * r1 - l2 * r2 - l3 * r3 - l4 * r4 - l5 * r5 - l6 * r6 + l7 * r7;
        self.c[1] = l0 * r1 + l1 * r0 - l2 * r4 - l3 * r5 + l4 * r2 + l5 * r3 - l6 * r7 - l7 * r6;
        self.c[2] = l0 * r2 + l1 * r4 + l2 * r0 - l3 * r6 - l4 * r1 + l5 * r7 + l6 * r3 + l7 * r5;
        self.c[3] = l0 * r3 + l1 * r5 + l2 * r6 + l3 * r0 - l4 * r7 - l5 * r1 - l6 * r2 - l7 * r4;
        self.c[4] = l0 * r4 - l1 * r2 + l2 * r1 - l3 * r7 + l4 * r0 - l5 * r6 + l6 * r5 - l7 * r3;
        self.c[5] = l0 * r5 - l1 * r3 + l2 * r7 + l3 * r1 + l4 * r6 + l5 * r0 - l6 * r4 + l7 * r2;
        self.c[6] = l0 * r6 - l1 * r7 - l2 * r3 + l3 * r2 - l4 * r5 + l5 * r4 + l6 * r0 - l7 * r1;
        self.c[7] = l0 * r7 + l1 * r6 - l2 * r5 + l3 * r4 + l4 * r3 - l5 * r2 + l6 * r1 + l7 * r0;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Trivector4 {
    c: [f32; 4],
}

impl Trivector4 {
    pub fn norm_squared(&self) -> f32 {
        self.c.iter().map(|&x| x * x).sum()
    }
}

impl Index<usize> for Trivector4 {
    type Output = f32;

    fn index(&self, i: usize) -> &f32 {
        &self.c[i]
    }
}

pub trait Wedge<T> {
    type Output;

    fn wedge(&self, rhs: T) -> Self::Output;
}

impl Wedge<na::Vector4> for na::Vector4 {
    type Output = Bivector4;

    fn wedge(&self, rhs: na::Vector4) -> Bivector4 {
        Bivector4 {
            c: [
                self[0] * rhs[1] - self[1] * rhs[0],
                self[0] * rhs[2] - self[2] * rhs[0],
                self[0] * rhs[3] - self[3] * rhs[0],
                self[1] * rhs[2] - self[2] * rhs[1],
                self[1] * rhs[3] - self[3] * rhs[1],
                self[2] * rhs[3] - self[3] * rhs[2],
            ],
        }
    }
}

impl Wedge<Bivector4> for na::Vector4 {
    type Output = Trivector4;

    fn wedge(&self, rhs: Bivector4) -> Trivector4 {
        Trivector4 {
            c: [
                self[0] * rhs[3] - self[1] * rhs[1] + self[2] * rhs[0],
                self[0] * rhs[4] - self[1] * rhs[2] + self[3] * rhs[0],
                self[0] * rhs[5] - self[2] * rhs[2] + self[3] * rhs[1],
                self[1] * rhs[5] - self[2] * rhs[4] + self[3] * rhs[3],
            ],
        }
    }
}

pub trait Reject {
    fn reject(&self, v: na::Vector4) -> na::Vector4;
}

impl Reject for na::Vector4 {
    #[rustfmt::skip]
    fn reject(&self, v: na::Vector4) -> na::Vector4 {
        na::vec4(
            v[0] * self[1] * self[1]
                + v[0] * self[2] * self[2]
                + v[0] * self[3] * self[3]
                - v[1] * self[0] * self[1]
                - v[2] * self[0] * self[2]
                - v[3] * self[0] * self[3],
            -v[0] * self[0] * self[1]
                + v[1] * self[0] * self[0]
                + v[1] * self[2] * self[2]
                + v[1] * self[3] * self[3]
                - v[2] * self[1] * self[2]
                - v[3] * self[1] * self[3],
            -v[0] * self[0] * self[2]
                - v[1] * self[1] * self[2]
                + v[2] * self[0] * self[0]
                + v[2] * self[1] * self[1]
                + v[2] * self[3] * self[3]
                - v[3] * self[2] * self[3],
            -v[0] * self[0] * self[3]
                - v[1] * self[1] * self[3]
                - v[2] * self[2] * self[3]
                + v[3] * self[0] * self[0]
                + v[3] * self[1] * self[1]
                + v[3] * self[2] * self[2],
        ) / self.norm_squared()
    }
}

impl Reject for Bivector4 {
    #[rustfmt::skip]
    fn reject(&self, v: na::Vector4) -> na::Vector4 {
        na::vec4(
            v[0] * self[3] * self[3]
                + v[0] * self[4] * self[4]
                + v[0] * self[5] * self[5]
                - v[1] * self[1] * self[3]
                - v[1] * self[2] * self[4]
                + v[2] * self[0] * self[3]
                - v[2] * self[2] * self[5]
                + v[3] * self[0] * self[4]
                + v[3] * self[1] * self[5],
            -v[0] * self[1] * self[3]
                - v[0] * self[2] * self[4]
                + v[1] * self[1] * self[1]
                + v[1] * self[2] * self[2]
                + v[1] * self[5] * self[5]
                - v[2] * self[0] * self[1]
                - v[2] * self[4] * self[5]
                - v[3] * self[0] * self[2]
                + v[3] * self[3] * self[5],
            v[0] * self[0] * self[3]
                - v[0] * self[2] * self[5]
                - v[1] * self[0] * self[1]
                - v[1] * self[4] * self[5]
                + v[2] * self[0] * self[0]
                + v[2] * self[2] * self[2]
                + v[2] * self[4] * self[4]
                - v[3] * self[1] * self[2]
                - v[3] * self[3] * self[4],
            v[0] * self[0] * self[4]
                + v[0] * self[1] * self[5]
                - v[1] * self[0] * self[2]
                + v[1] * self[3] * self[5]
                - v[2] * self[1] * self[2]
                - v[2] * self[3] * self[4]
                + v[3] * self[0] * self[0]
                + v[3] * self[1] * self[1]
                + v[3] * self[3] * self[3],
        ) / self.norm_squared()
    }
}

impl Reject for Trivector4 {
    #[rustfmt::skip]
    fn reject(&self, v: na::Vector4) -> na::Vector4 {
        na::vec4(
            self[3] * ( v[0] * self[3] - v[1] * self[2] + v[2] * self[1] - v[3] * self[0]),
            self[2] * (-v[0] * self[3] + v[1] * self[2] - v[2] * self[1] + v[3] * self[0]),
            self[1] * ( v[0] * self[3] - v[1] * self[2] + v[2] * self[1] - v[3] * self[0]),
            self[0] * (-v[0] * self[3] + v[1] * self[2] - v[2] * self[1] + v[3] * self[0]),
        ) / self.norm_squared()
    }
}
