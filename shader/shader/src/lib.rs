#![no_std]

use core::f32::consts::{PI, TAU};
use push_constants::shader::*;
use shared::complex::Complex;
use shared::*;
use spirv_std::glam::*;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;
use spirv_std::spirv;

mod palette;
mod sdf;

pub fn lerp(x: f32, y: f32, a: f32) -> f32 {
    x * (1.0 - a) + y * a
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    // Scale, bias and saturate x to 0..1 range
    let x = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    // Evaluate polynomial
    x * x * (3.0 - 2.0 * x)
}

fn get_col(palette: Palette, x: f32) -> Vec3 {
    match palette {
        Palette::A => palette::cola(x),
        Palette::B => palette::colb(x),
        Palette::C => palette::colc(x),
        Palette::D => palette::cold(x),
        Palette::E => palette::cole(x),
        Palette::F => palette::colf(x),
    }
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[cfg(not(feature = "emulate_constants"))]
    #[spirv(push_constant)]
    constants: &FragmentConstants,
    #[cfg(feature = "emulate_constants")]
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)]
    constants: &FragmentConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] iteration_points: &[Vec2],
    output: &mut Vec4,
) {
    let size = constants.size.as_vec2();
    let is_split_vertical = size.x > size.y;
    let n = if is_split_vertical { Vec2::X } else { Vec2::Y };
    let render_julia_set = constants.render_julia_set.into();
    let mandelbrot_zoom = constants.mandelbrot_camera_zoom;
    let mandelbrot_uv = (frag_coord.xy() - 0.5 * size) / size.y / mandelbrot_zoom
        + constants.mandelbrot_camera_translate;
    let is_julia =
        render_julia_set && frag_coord.xy().dot(n) > size.dot(n) * constants.render_split;

    let mut col = {
        let (z0, c): (Complex, Complex) = if is_julia {
            (
                ((frag_coord.xy() - 0.5 * size) / size.y / constants.julia_camera_zoom
                    + constants.julia_camera_translate)
                    .into(),
                constants.marker.into(),
            )
        } else {
            (Complex::ZERO, mandelbrot_uv.into())
        };
        match constants.render_style {
            RenderStyle::Iterations => col_from_iterations(constants, z0, c),
            RenderStyle::Arg => col_from_arg(constants, z0, c),
            RenderStyle::LastDistance => col_from_last_distance(constants, z0, c),
            RenderStyle::TotalDistance => col_from_total_distance(constants, z0, c),
        }
    };

    // Slider
    if render_julia_set {
        let d = (frag_coord.xy() + 0.5 - size * constants.render_split * n)
            .dot(n)
            .abs();
        let intensity = smoothstep(4.0, 0.0, d);
        col += Vec3::ONE * intensity;
    }

    let show_iterations = constants.show_iterations.into();
    if (show_iterations || render_julia_set) && !is_julia {
        // Iteration line segments
        if show_iterations {
            let mut intensity: f32 = 0.0;
            for i in 0..constants.num_points as usize - 1 {
                let p0 = iteration_points[i];
                let p1 = iteration_points[i + 1];
                let d = sdf::line_segment(mandelbrot_uv, p0, p1).abs();
                intensity = intensity.max(smoothstep(2.0 / mandelbrot_zoom / size.y, 0.0, d).abs());
            }
            col += intensity;
        }
        // Marker
        {
            let d = sdf::disk(
                mandelbrot_uv - constants.marker,
                MARKER_RADIUS / mandelbrot_zoom / size.y,
            );
            let intensity = smoothstep(3.0 / mandelbrot_zoom / size.y, 0.0, d.abs());
            if d < 0.0 {
                col = Vec3::splat(intensity);
            } else {
                col += intensity;
            }
        }
    }

    *output = col.extend(1.0);
}

fn col_from_iterations(constants: &FragmentConstants, mut z: Complex, c: Complex) -> Vec3 {
    let num_iters = constants.num_iterations;
    let mut i = 0;
    let mut norm_sq = z.norm_squared();
    let mut prev_norm_sq = 0.0;
    while norm_sq < 4.0 && i < num_iters as u32 {
        z = z * z + c;
        i += 1;
        prev_norm_sq = norm_sq;
        norm_sq = z.norm_squared();
    }
    let h = get_lerp_factor(prev_norm_sq, norm_sq);
    if norm_sq < 4.0 || i == num_iters as u32 && constants.num_iterations.fract() < h {
        Vec3::ZERO
    } else {
        let period = constants.palette_period * 0.2;
        let t = constants.animate_time;
        let s = smoothstep(0.0, constants.smooth_factor, h);
        let x = (i as f32 + s) * period - t;
        get_col(constants.palette, x)
    }
}

fn col_from_arg(constants: &FragmentConstants, mut z: Complex, c: Complex) -> Vec3 {
    let num_iters = constants.num_iterations;
    let mut i = 0;
    let mut norm_sq = z.norm_squared();
    let mut prev_z = Complex::ZERO;
    while norm_sq < 4.0 && i < num_iters as u32 {
        prev_z = z;
        z = z * z + c;
        i += 1;
        norm_sq = z.norm_squared();
    }
    let h = get_lerp_factor(prev_z.norm_squared(), norm_sq);
    if norm_sq < 4.0 || i == num_iters as u32 && constants.num_iterations.fract() < h {
        Vec3::ZERO
    } else {
        let period = (1 << (constants.palette_period * 3.0) as u32) as f32 / TAU;
        let t = constants.animate_time * PI;
        let col = get_col(constants.palette, prev_z.arg() * period - t);
        let col2 = get_col(constants.palette, z.arg() * period - t);
        let s = smoothstep(0.0, constants.smooth_factor, h);
        col.lerp(col2, s)
    }
}

fn col_from_last_distance(constants: &FragmentConstants, mut z: Complex, c: Complex) -> Vec3 {
    let num_iters = constants.num_iterations;
    let mut i = 0;
    let mut dist_sq = 0.0;
    let mut prev_dist_sq = 0.0;
    let mut norm_sq = z.norm_squared();
    let mut prev_z = Complex::ZERO;
    while norm_sq < 4.0 && i < num_iters as u32 {
        prev_z = z;
        z = z * z + c;
        prev_dist_sq = dist_sq;
        dist_sq = prev_z.distance_squared(z.0);
        i += 1;
        norm_sq = z.norm_squared();
    }
    let h = get_lerp_factor(prev_z.norm_squared(), norm_sq);
    if norm_sq < 4.0 || i == num_iters as u32 && constants.num_iterations.fract() < h {
        Vec3::ZERO
    } else {
        let period = constants.palette_period;
        let t = constants.animate_time;
        let s = smoothstep(0.0, constants.smooth_factor, h);
        let prev_dist = prev_dist_sq.sqrt();
        let dist = dist_sq.sqrt();
        let col = get_col(constants.palette, prev_dist * period - t);
        let col2 = get_col(constants.palette, dist * period - t);
        col.lerp(col2, s)
    }
}

fn col_from_total_distance(constants: &FragmentConstants, mut z: Complex, c: Complex) -> Vec3 {
    let num_iters = constants.num_iterations;
    let mut i = 0;
    let mut dist = 0.0;
    let mut prev_dist = 0.0;
    let mut norm_sq = z.norm_squared();
    let mut prev_z = Complex::ZERO;
    while norm_sq < 4.0 && i < num_iters as u32 {
        prev_z = z;
        z = z * z + c;
        prev_dist = dist;
        dist += prev_z.distance(z.0);
        i += 1;
        norm_sq = z.norm_squared();
    }
    let h = get_lerp_factor(prev_z.norm_squared(), norm_sq);
    if norm_sq < 4.0 || i == num_iters as u32 && constants.num_iterations.fract() < h {
        Vec3::ZERO
    } else {
        let period = constants.palette_period;
        let t = constants.animate_time;
        let col = get_col(constants.palette, prev_dist * period - t);
        let col2 = get_col(constants.palette, dist * period - t);
        let s = smoothstep(0.0, constants.smooth_factor, h);
        col.lerp(col2, s)
    }
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    let uv = vec2(((vert_id << 1) & 2) as f32, (vert_id & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;
    *out_pos = pos.extend(0.0).extend(1.0);
}
