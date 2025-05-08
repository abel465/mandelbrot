#![no_std]

use push_constants::shader::*;
use shared::complex::Complex;
use shared::*;
use spirv_std::glam::*;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

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

fn from_uv(constants: &FragmentConstants, p: Vec2) -> Vec2 {
    let size = constants.size.as_vec2();
    (p - constants.mandelbrot_camera_translate) / vec2(size.x / size.y, 1.0)
        * constants.mandelbrot_camera_zoom
        * size
        + 0.5 * size
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
    let num_iters = constants.num_iterations;
    let mut col = match constants.style {
        RenderStyle::RedGlow => style_red_glow(z0, c, num_iters),
        RenderStyle::Circus => style_circus(z0, c, num_iters),
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
                let d = sdf::line_segment(frag_coord.xy(), p0, p1).abs();
                intensity = intensity.max(smoothstep(2.0, 0.0, d).abs());
            }
            col += intensity;
        }
        // Marker
        {
            let d = sdf::disk(
                frag_coord.xy() - from_uv(&constants, constants.marker),
                MARKER_RADIUS,
            );
            let intensity = smoothstep(3.0, 0.0, d.abs());
            if d < 0.0 {
                col = Vec3::splat(intensity);
            } else {
                col += intensity;
            }
        }
    }

    *output = col.extend(1.0);
}

fn style_circus(z0: Complex, c: Complex, num_iters: f32) -> Vec3 {
    let num_iter_fract = num_iters.fract();
    let num_iters = num_iters as u32;

    let mut z = z0;
    let mut i = 0;

    let col = loop {
        z = z * z + c;
        i += 1;

        let norm_squared = z.norm_squared();
        if norm_squared >= 4.0 {
            let col = if i & 1 == 1 {
                Vec3::X + Vec3::Y
            } else {
                Vec3::X
            };
            let col = col * (10.0 / norm_squared).sin();

            if i >= num_iters {
                let show = ((norm_squared - 4.0) * num_iter_fract) > 0.3;
                if !show {
                    break Vec3::ZERO;
                }
            }

            break col;
        }

        if i >= num_iters {
            return Vec3::ZERO;
        }
    };

    col
}

fn style_red_glow(z0: Complex, c: Complex, num_iters: f32) -> Vec3 {
    let num_iter_fract = num_iters.fract();
    let num_iters = num_iters as u32;

    let mut z = z0;
    let mut i = 0;
    loop {
        z = z * z + c;
        i += 1;

        let norm_squared = z.norm_squared();
        if norm_squared >= 4.0 {
            let smoothing = 0.03 * smoothstep(1.0, 0.0, norm_squared.sqrt() - 2.0);
            let red = i as f32 / num_iters as f32 + smoothing;

            if i >= num_iters {
                let show = ((norm_squared - 4.0) * num_iter_fract) > 0.3;
                if !show {
                    break Vec3::ZERO;
                }
            }

            break red * Vec3::X;
        }

        if i >= num_iters {
            break Vec3::ZERO;
        }
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
