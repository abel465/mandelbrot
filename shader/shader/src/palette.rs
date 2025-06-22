use core::f32::consts::TAU;
use spirv_std::glam::*;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

pub fn pal(t: f32, a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Vec3 {
    a + b * cos_vec3(TAU * (c * t + d))
}

fn cos_vec3(v: Vec3) -> Vec3 {
    vec3(v.x.cos(), v.y.cos(), v.z.cos())
}

pub fn zebra(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(1.0, 1.0, 1.0),
        vec3(0.0, 0.0, 0.0),
    )
}

pub fn rgb(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(1.0, 1.0, 1.0),
        vec3(0.0, 0.33, 0.67),
    )
}

pub fn neon_a(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(1.0, 1.0, 0.5),
        vec3(0.8, 0.90, 0.30),
    )
}

pub fn neon_b(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.6, 0.6, 0.6),
        vec3(0.3, 0.3, 0.3),
        vec3(1.0, 2.0, 0.5),
        vec3(0.5, 0.0, 0.67),
    )
}

pub fn neon_c(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.6, 0.6, 0.6),
        vec3(0.3, 0.3, 0.3),
        vec3(0.5, 2.0, 1.0),
        vec3(0.0, 0.5, 0.67),
    )
}

pub fn pastel(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.9, 0.8, 0.8),
        vec3(0.5, 0.2, 0.3),
        vec3(1.0, 2.0, 1.0),
        vec3(0.0, 0.5, 0.67),
    )
}

pub fn copper(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.66, 0.6, 0.6),
        vec3(0.3, 0.3, 0.3),
        vec3(1.0, 1.0, 1.0),
        vec3(0.33, 0.20, 0.20),
    )
}

pub fn red_and_black(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.5, 0.0, 0.0),
        vec3(0.5, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 0.0),
    )
}

pub fn solarized_dark(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(2.0, 1.0, 0.0),
        vec3(0.5, 0.20, 0.25),
    )
}

pub fn highlighter(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.8, 0.5, 0.4),
        vec3(0.2, 0.4, 0.2),
        vec3(2.0, 1.0, 1.0),
        vec3(0.0, 0.333, 0.667),
    )
}
