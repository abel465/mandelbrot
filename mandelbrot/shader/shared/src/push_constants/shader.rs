use super::*;
use bytemuck::NoUninit;

#[derive(Copy, Clone, Debug, NoUninit)]
#[repr(C)]
pub struct FragmentConstants {
    pub size: Size,
    pub translate: Vec2,
    pub cursor: Vec2,
    pub prev_cursor: Vec2,
    pub time: f32,
    pub mouse_button_pressed: u32,
    pub camera_translate: Vec2,
    pub camera_zoom: f32,
    pub debug: Bool,
}

#[derive(Copy, Clone, Debug, NoUninit)]
#[repr(C)]
pub struct ComputeConstants {
    pub size: Size,
    pub time: f32,
    pub zoom: f32,
    pub transition: Bool,
}
