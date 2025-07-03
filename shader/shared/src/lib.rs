#![cfg_attr(target_arch = "spirv", no_std)]

pub mod complex;
pub mod grid;
pub mod push_constants;

use glam::*;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

pub const MARKER_RADIUS: f32 = 8.0;
pub const GRID_SIZE: UVec2 = uvec2(2880, 1620);

// Given lerp(x, y, a) = e, x < e, y >= e
// Returns 'a' which is a value between 0 and 1
pub fn get_proximity(x: f32, y: f32, e: f32) -> f32 {
    f32::inverse_lerp(x, y, e).sqrt()
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    // Scale, bias and saturate x to 0..1 range
    let x = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    // Evaluate polynomial
    x * x * (3.0 - 2.0 * x)
}
