#![cfg_attr(target_arch = "spirv", no_std)]

pub mod grid;
pub mod push_constants;

use glam::*;

pub const DIM: UVec2 = UVec2::splat(192);

#[derive(Clone, Copy, Default, bytemuck::NoUninit)]
#[repr(u32)]
pub enum CellState {
    #[default]
    Off,
    On,
    Dying,
    Spawning,
}
