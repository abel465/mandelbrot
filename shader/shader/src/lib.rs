#![no_std]

use push_constants::shader::*;
use shared::complex::Complex;
use shared::*;
use spirv_std::glam::*;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
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
        let num_iters = constants.num_iterations;
        let (mut z, c): (Complex, Complex) = if is_julia {
            (
                ((frag_coord.xy() - 0.5 * size) / size.y / constants.julia_camera_zoom
                    + constants.julia_camera_translate)
                    .into(),
                constants.marker.into(),
            )
        } else {
            (Complex::ZERO, mandelbrot_uv.into())
        };
        let mut i = 0;
        let mut norm_squared;
        loop {
            z = z * z + c;
            i += 1;
            norm_squared = z.norm_squared();
            if i >= num_iters as u32 {
                break;
            }
            if norm_squared >= 4.0 {
                break;
            }
        }
        if i >= num_iters as u32
            && ((norm_squared - 4.0) * 0.1) < (1.0 - constants.num_iterations.fract())
        {
            Vec3::ZERO
        } else {
            let period = constants.palette_period;
            let t = constants.animate_time * 10.0 / period;
            let x = smoothstep(constants.smooth_factor, 0.0, norm_squared - 4.0);
            let x = ((i as f32 + x - t) * 0.1 * period).fract();
            match constants.palette {
                Palette::A => palette::cola(x),
                Palette::B => palette::colb(x),
                Palette::C => palette::colc(x),
                Palette::D => palette::cold(x),
                Palette::E => palette::cole(x),
                Palette::F => palette::colf(x),
            }
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

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    let uv = vec2(((vert_id << 1) & 2) as f32, (vert_id & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;
    *out_pos = pos.extend(0.0).extend(1.0);
}
