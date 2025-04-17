#![no_std]

use complex::Complex;
use push_constants::shader::*;
use shared::*;
use spirv_std::glam::*;
use spirv_std::num_traits::Float;
use spirv_std::spirv;

mod complex;

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
    output: &mut Vec4,
) {
    let size = constants.size.as_vec2();
    let uv: Complex = (1.0 / constants.camera_zoom * (frag_coord.xy() - 0.5 * size) / size.y
        + constants.camera_translate)
        .into();

    let col = match constants.style {
        RenderStyle::RedGlow => style_red_glow(uv, constants),
        RenderStyle::Circus => style_circus(uv, constants),
    };
    *output = col.extend(1.0)
}

fn style_circus(uv: Complex, constants: &FragmentConstants) -> Vec3 {
    let fnum_iters = constants.num_iterations;
    let num_iters = fnum_iters as u32;
    let num_iter_fract = fnum_iters.fract();

    let mut z = Complex::ZERO;
    let mut i = 0;

    let col = loop {
        z = z * z + uv;
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

fn style_red_glow(uv: Complex, constants: &FragmentConstants) -> Vec3 {
    let fnum_iters = constants.num_iterations;
    let num_iters = fnum_iters as u32;
    let num_iter_fract = fnum_iters.fract();

    let mut z = Complex::ZERO;
    let mut i = 0;
    loop {
        z = z * z + uv;
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
