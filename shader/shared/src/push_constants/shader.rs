use super::*;
use bytemuck::NoUninit;

#[derive(Copy, Clone, Debug, NoUninit, Default, PartialEq)]
#[repr(u32)]
pub enum Palette {
    #[default]
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(Copy, Clone, Debug, NoUninit)]
#[repr(C)]
pub struct FragmentConstants {
    pub cursor: Vec2,
    pub prev_cursor: Vec2,
    pub mandelbrot_camera_translate: Vec2,
    pub julia_camera_translate: Vec2,
    pub size: Size,
    pub marker: Vec2,
    pub time: f32,
    pub mandelbrot_camera_zoom: f32,
    pub julia_camera_zoom: f32,
    pub num_iterations: f32,
    pub show_iterations: Bool,
    pub num_points: u32,
    pub render_julia_set: Bool,
    pub render_split: f32,
    pub palette: Palette,
    pub smooth_factor: f32,
    pub animate_time: f32,
    pub padding: u32,
}
