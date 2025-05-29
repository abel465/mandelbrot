#![cfg_attr(target_arch = "spirv", no_std)]

pub mod complex;
pub mod push_constants;

pub const MARKER_RADIUS: f32 = 8.0;

// Given lerp(x, y, a) = 4, x < 4, y >= 4
// Returns 'a' which is a value between 0 and 1
pub fn get_lerp_factor(x: f32, y: f32) -> f32 {
    (4.0 - x) / (y - x)
}
