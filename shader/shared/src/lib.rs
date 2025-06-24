#![cfg_attr(target_arch = "spirv", no_std)]

pub mod complex;
pub mod grid;
pub mod push_constants;

use glam::*;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

pub const MARKER_RADIUS: f32 = 8.0;
pub const GRID_SIZE: UVec2 = uvec2(2880, 1620);

// Given lerp(x, y, a) = 2, x < 2, y >= 2
// Returns 'a' which is a value between 0 and 1
pub fn get_proximity(x: f32, y: f32) -> f32 {
    ((2.0 - x) / (y - x)).sqrt()
}
