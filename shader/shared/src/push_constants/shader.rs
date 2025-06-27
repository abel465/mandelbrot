use super::*;
use crate::*;
use bytemuck::NoUninit;

#[derive(Copy, Clone, Debug, NoUninit, Default, PartialEq)]
#[repr(u32)]
pub enum RenderStyle {
    #[default]
    Iterations,
    FinalAngle,
    FinalDistance,
    DistanceSum,
    NormSum,
}

#[derive(Copy, Clone, Debug, NoUninit, Default, PartialEq)]
#[repr(u32)]
pub enum Palette {
    #[default]
    Pastel,
    Zebra,
    Copper,
    SolarizedDark,
    Highlighter,
    RedAndBlack,
    RGB,
    NeonA,
    NeonB,
    NeonC,
}

#[derive(Copy, Clone, Debug, NoUninit, Default)]
#[repr(C)]
pub struct RenderParameters {
    pub i: u32,
    pub x: f32,
}

impl RenderParameters {
    pub fn new(
        constants: &FragmentConstants,
        inside: bool,
        i: u32,
        h: f32,
        x0: f32,
        x1: f32,
    ) -> Self {
        let i = if (inside && constants.render_partitioning == RenderPartitioning::Outside)
            || (!inside && constants.render_partitioning == RenderPartitioning::Inside)
        {
            core::u32::MAX
        } else {
            i
        };
        let s = if inside {
            constants.num_iterations.fract() * constants.smooth_factor
        } else {
            smoothstep(0.0, constants.smooth_factor, h)
        };
        let x = x0.lerp(x1, s);

        Self { i, x }
    }
}

#[derive(Copy, Clone, Debug, NoUninit, Default, PartialEq)]
#[repr(u32)]
pub enum IterationMode {
    #[default]
    Regular,
    Perturbation,
}

#[derive(Copy, Clone, Debug, NoUninit, Default, PartialEq)]
#[repr(u32)]
pub enum RenderPartitioning {
    #[default]
    Outside,
    Inside,
    Both,
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
    pub iteration_mode: IterationMode,
    pub render_partitioning: RenderPartitioning,
}
