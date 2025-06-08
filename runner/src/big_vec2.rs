use dashu::float::FBig;
use std::ops::*;

#[derive(Clone, Debug)]
pub struct BigVec2 {
    pub x: FBig,
    pub y: FBig,
}

impl BigVec2 {
    pub const ZERO: Self = Self::new(FBig::ZERO, FBig::ZERO);

    pub const fn new(x: FBig, y: FBig) -> Self {
        Self { x, y }
    }

    #[allow(dead_code)]
    pub fn precision(&self) -> glam::UVec2 {
        let p0 = self.x.precision();
        let p1 = self.y.precision();
        glam::uvec2(p0 as u32, p1 as u32)
    }

    pub fn with_precision(mut self, precision: usize) -> Self {
        self.x = self.x.with_precision(precision).value();
        self.y = self.y.with_precision(precision).value();
        self
    }

    pub fn from_dvec2(v: glam::DVec2) -> Self {
        let x = FBig::try_from(v.x).unwrap();
        let y = FBig::try_from(v.y).unwrap();
        Self { x, y }
    }

    pub fn from_f64s(x: f64, y: f64) -> Self {
        let x = FBig::try_from(x).unwrap();
        let y = FBig::try_from(y).unwrap();
        Self { x, y }
    }

    pub fn as_vec2(&self) -> glam::Vec2 {
        glam::vec2(self.x.to_f32().value(), self.y.to_f32().value())
    }

    pub fn as_dvec2(&self) -> glam::DVec2 {
        glam::dvec2(self.x.to_f64().value(), self.y.to_f64().value())
    }
}

impl Add for BigVec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for BigVec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl AddAssign for BigVec2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl SubAssign for BigVec2 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

// impl AddAssign<glam::Vec2> for Vec2Big {
//     fn add_assign(&mut self, other: glam::Vec2) {
//         self.x += FBig::try_from(other.x).unwrap();
//         self.y += FBig::try_from(other.y).unwrap();
//     }
// }

// impl SubAssign<glam::Vec2> for Vec2Big {
//     fn sub_assign(&mut self, other: glam::Vec2) {
//         self.x -= FBig::try_from(other.x).unwrap();
//         self.y -= FBig::try_from(other.y).unwrap();
//     }
// }

impl AddAssign<glam::DVec2> for BigVec2 {
    fn add_assign(&mut self, other: glam::DVec2) {
        self.x += FBig::try_from(other.x).unwrap();
        self.y += FBig::try_from(other.y).unwrap();
    }
}

impl SubAssign<glam::DVec2> for BigVec2 {
    fn sub_assign(&mut self, other: glam::DVec2) {
        self.x -= FBig::try_from(other.x).unwrap();
        self.y -= FBig::try_from(other.y).unwrap();
    }
}

impl DivAssign<f64> for BigVec2 {
    fn div_assign(&mut self, other: f64) {
        self.x /= FBig::try_from(other).unwrap();
        self.y /= FBig::try_from(other).unwrap();
    }
}

impl Div<f64> for BigVec2 {
    type Output = Self;
    fn div(mut self, other: f64) -> Self::Output {
        self /= other;
        self
    }
}

impl std::fmt::Display for BigVec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Vec2Big(")?;
        self.x.fmt(f)?;
        write!(f, ", ")?;
        self.y.fmt(f)?;
        write!(f, ")")
    }
}
