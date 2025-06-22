use super::*;
use bytemuck::NoUninit;

#[derive(Copy, Clone, Debug, NoUninit, Default, PartialEq)]
#[repr(u32)]
pub enum RenderStyle {
    #[default]
    Iterations,
    Arg,
    LastDistance,
    TotalDistance,
    NormSum,
}

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
}

#[derive(Copy, Clone, Debug, NoUninit, Default)]
#[repr(C)]
pub struct RenderParameters {
    pub inside: Bool,
    pub x0: f32,
    pub x1: f32,
    pub h: f32,
}

impl RenderParameters {
    pub fn new(inside: bool, x0: f32, x1: f32, h: f32) -> Self {
        Self {
            inside: inside.into(),
            x0,
            x1,
            h,
        }
    }
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
    pub marker_screen_space: Vec2,
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
    pub palette_period: f32,
    pub render_style: RenderStyle,
    pub mandelbrot_num_ref_iterations: u32,
    pub needs_reiterate_mandelbrot: Bool,
    pub needs_reiterate_julia: Bool,
}
