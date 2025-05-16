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

pub fn cola(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(1.0, 1.0, 1.0),
        vec3(0.0, 0.33, 0.67),
    )
}

pub fn colb(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(1.0, 1.0, 1.0),
        vec3(0.0, 0.10, 0.20),
    )
}

pub fn colc(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(1.0, 1.0, 1.0),
        vec3(0.3, 0.20, 0.20),
    )
}

pub fn cold(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(1.0, 1.0, 0.5),
        vec3(0.8, 0.90, 0.30),
    )
}

pub fn cole(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(2.0, 1.0, 0.0),
        vec3(0.5, 0.20, 0.25),
    )
}

pub fn colf(t: f32) -> Vec3 {
    pal(
        t,
        vec3(0.8, 0.5, 0.4),
        vec3(0.2, 0.4, 0.2),
        vec3(2.0, 1.0, 1.0),
        vec3(0.0, 0.25, 0.25),
    )
}
