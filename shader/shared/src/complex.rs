use core::convert::From;
use core::ops::*;
use spirv_std::glam::Vec2;
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;

#[derive(Copy, Clone)]
pub struct Complex(pub Vec2);

impl Complex {
    pub const fn new(x: f32, y: f32) -> Self {
        Complex(Vec2::new(x, y))
    }
    pub const ZERO: Complex = Complex(Vec2::ZERO);
    pub const ONE: Complex = Complex(Vec2::X);
    pub const I: Complex = Complex(Vec2::Y);
}

impl Complex {
    pub fn conjugate(&self) -> Self {
        Self::new(self.x, -self.y)
    }

    pub fn powf(self, exp: f32) -> Self {
        if exp == 0.0 {
            Complex::ONE
        } else {
            let (r, theta) = self.to_polar();
            Self::from_polar(r.powf(exp), theta * exp)
        }
    }

    pub fn norm(self) -> f32 {
        self.length()
    }

    pub fn norm_squared(self) -> f32 {
        self.length_squared()
    }

    pub fn arg(self) -> f32 {
        self.y.atan2(self.x)
    }

    pub fn to_polar(self) -> (f32, f32) {
        (self.norm(), self.arg())
    }

    pub fn from_polar(r: f32, theta: f32) -> Self {
        r * Complex::from_angle(theta)
    }

    pub fn sqrt(self) -> Self {
        Self::new(
            ((self.norm() + self.x) / 2.0).sqrt(),
            self.y.signum() * ((self.norm() - self.x) / 2.0).sqrt(),
        )
    }

    pub fn exp(self) -> Self {
        Self::from_polar(self.x.exp(), self.y)
    }

    pub fn from_angle(angle: f32) -> Self {
        Complex(Vec2::from_angle(angle))
    }
}

impl From<Vec2> for Complex {
    fn from(value: Vec2) -> Self {
        Complex(value)
    }
}

impl From<f32> for Complex {
    fn from(value: f32) -> Complex {
        Complex::new(value, 0.0)
    }
}

impl Deref for Complex {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Complex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Add for Complex {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Complex::new(self.x + other.x, self.y + other.y)
    }
}

impl Add<f32> for Complex {
    type Output = Self;
    fn add(self, x: f32) -> Self::Output {
        Complex::new(self.x + x, self.y)
    }
}

impl Add<Complex> for f32 {
    type Output = Complex;
    fn add(self, z: Complex) -> Self::Output {
        Complex::new(self + z.x, z.y)
    }
}

impl Sub for Complex {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Complex::new(self.x - other.x, self.y - other.y)
    }
}

impl AddAssign for Complex {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl SubAssign for Complex {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Mul for Complex {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Complex::new(
            self.x * other.x - self.y * other.y,
            self.x * other.y + self.y * other.x,
        )
    }
}

impl Mul<f32> for Complex {
    type Output = Self;
    fn mul(self, other: f32) -> Self::Output {
        Complex::new(self.x * other, self.y * other)
    }
}

impl Mul<Complex> for f32 {
    type Output = Complex;
    fn mul(self, other: Complex) -> Self::Output {
        Complex::new(self * other.x, self * other.y)
    }
}

impl Div<f32> for Complex {
    type Output = Self;
    fn div(self, other: f32) -> Self::Output {
        Complex::new(self.x / other, self.y / other)
    }
}

impl Div<Complex> for Complex {
    type Output = Self;
    fn div(self, other: Complex) -> Self::Output {
        let d = other.x * other.x + other.y * other.y;
        Complex::new(
            (self.x * other.x + self.y * other.y) / d,
            (self.y * other.x - self.x * other.y) / d,
        )
    }
}

impl Neg for Complex {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Complex::new(-self.x, -self.y)
    }
}

// #[cfg(not(target_arch = "spirv"))]
// impl std::fmt::Display for Complex {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         if self.y == 0.0 {
//             write!(f, "{}", self.x)
//         } else if self.y < 0.0 {
//             write!(f, "{} - {}i", self.x, -self.y)
//         } else {
//             write!(f, "{} + {}i", self.x, self.y)
//         }
//     }
// }
//
// #[cfg(not(target_arch = "spirv"))]
// impl std::fmt::Debug for Complex {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "Complex({}, {})", self.x, self.y)
//     }
// }
