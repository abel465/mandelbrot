#![no_std]

use complex::Complex;
use grid::GridRefMut;
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
    let coord = frag_coord.xy() - constants.translate;
    // let coord = vec2(coord.x, -coord.y);
    let size = constants.size.as_vec2();
    // let size = vec2(size.x, -size.y);
    let uv: Complex = (1.0 / constants.camera_zoom * (coord - 0.5 * size) / size.y
        + constants.camera_translate)
        .into();

    let col = if constants.style = 

        RenderStyle::Yellow 
    {
        style_yellow(uv, constants)
    } else {
        style_red(uv,constants)

    };
    *output = col.extend(1.0)

}
fn style_yellow(uv: Complex, constants: &FragmentConstants) -> Vec3 {
    let fnum_iters = constants.num_iterations; // + ((constants.time * 10.0).sin() + 1.0) / 2.0 * 0.2;
    let num_iters = fnum_iters as u32;
    let num_iter_fract = fnum_iters.fract();

    let mut z = Complex::ZERO;
    let mut i = 0;
    let mut norm_squared = 0.0;
    while i < num_iters {
        z = z * z + uv;
        i += 1;
        norm_squared = z.norm_squared();
        if norm_squared >= 4.0 {
            break;
        }
    }

    let x = norm_squared / 10.0;
    let bob = ((x.sin() + 1.0) / 2.0).max(0.7);
    let mut col = vec3(bob, 0.0, 0.0);
    if i & 1 == 1 {
        col.y = bob;
    }
    let mut output = Vec4::ZERO; 
    output = col.extend(1.0);
    if i == num_iters {
        if norm_squared >= 4.0 {
            let mut x = (norm_squared - 4.0) * num_iter_fract;
            if x < 0.33 {
                x = 0.0;
            }

            let mut col = vec3(x, 0.0, 0.0);
            if i & 1 == 1 {
                col.y = x;
            }
            output = col.extend(1.0);
        } else {
            output = vec3(0.0, 0.0, 0.0).extend(1.0);
        }
    }
    if i < 2 {
        let x = norm_squared / 50.0;
        let bob = ((x.sin() + 1.0) / 2.0).max(0.7);
        let mut col = vec3(bob, 0.0, 0.0);
        if i & 1 == 1 {
            col.y = bob;
        }
        output = col.extend(1.0);
    }
    if x.clamp(0.0, 10.0) < 0.45 && i < num_iters {
        output = vec3(1.0, 0.7, 0.0).extend(1.0);
    }
    vec3(output.x, output.y, output.z)

}

fn style_red(uv: Complex, constants: &FragmentConstants) -> Vec3 {
    let fnum_iters = constants.num_iterations;
    let num_iters = fnum_iters as u32;

    let mut z = Complex::ZERO;
    let mut i = 0;
    let mut norm_squared = 0.0;
    while i < num_iters {
        z = z * z + uv;
        i += 1;
        norm_squared = z.norm_squared();
        if norm_squared >= 4.0 {
            break;
        }
    }
    let gg = norm_squared.sqrt() - 2.0;
    let g = smoothstep(1.0, 0.0, gg) * 0.03;
    let o = i as f32;

    let mut col = Vec3::ZERO;
    col.x += o / fnum_iters;
    col.x += g;

    if i == num_iters {
        Vec3::ZERO
    } else {
        col
    }
}
