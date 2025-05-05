use spirv_std::glam::*;

fn project_onto_segment(a: Vec2, b: Vec2) -> Vec2 {
    b * (a.dot(b) / b.length_squared()).clamp(0.0, 1.0)
}

pub fn disk(p: Vec2, r: f32) -> f32 {
    p.length() - r
}

pub fn line_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    p.distance(a + project_onto_segment(p - a, b - a))
}
