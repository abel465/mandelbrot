use crate::big_vec2::BigVec2;
use dashu::float::FBig;
use std::ops::*;

#[derive(Clone, Debug)]
pub struct Complex(BigVec2);

impl Complex {
    pub const ZERO: Self = Self(BigVec2::ZERO);

    pub const fn new(x: FBig, y: FBig) -> Self {
        Self(BigVec2::new(x, y))
    }

    pub fn square(self) -> Self {
        const TWO: FBig = dashu::fbig!(10);
        Self::new(self.x.sqr() - self.y.sqr(), self.0.x * self.0.y * TWO)
    }

    pub fn norm_squared(&self) -> FBig {
        self.x.sqr() + self.y.sqr()
    }

    pub fn with_precision(self, precision: usize) -> Self {
        Self(self.0.with_precision(precision))
    }
}

impl Deref for Complex {
    type Target = BigVec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Complex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<BigVec2> for Complex {
    fn from(value: BigVec2) -> Self {
        Complex(value)
    }
}

impl Add for Complex {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl Sub for Complex {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}
