use super::*;
use bytemuck::NoUninit;

#[derive(Copy, Clone, Debug, NoUninit, Default, PartialEq)]
#[repr(u32)]
pub enum RenderStyle {
    #[default]
    RedGlow,
    Circus,
}

#[derive(Copy, Clone, Debug, NoUninit)]
#[repr(C)]
pub struct FragmentConstants {
    pub cursor: Vec2,
    pub prev_cursor: Vec2,
    pub camera_translate: Vec2,
    pub size: Size,
    pub time: f32,
    pub camera_zoom: f32,
    pub num_iterations: f32,
    pub style: RenderStyle,
}
