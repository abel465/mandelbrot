#![no_std]

use core::f32::consts::TAU;
use push_constants::shader::*;
use shared::complex::Complex;
use shared::grid::*;
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

trait Mandelbrot {
    fn z0(&self) -> Complex;
    fn iterate<F: FnMut(Complex)>(self, constants: &FragmentConstants, f: F) -> MandelbrotResult;
}

struct MandelbrotResult {
    inside: bool,
    i: u32,
    h: f32,
}

struct RegularMandelbrot {
    z0: Complex,
    c: Complex,
}

impl Mandelbrot for RegularMandelbrot {
    fn z0(&self) -> Complex {
        self.z0
    }

    fn iterate<F: FnMut(Complex)>(
        self,
        constants: &FragmentConstants,
        mut f: F,
    ) -> MandelbrotResult {
        let RegularMandelbrot { z0: mut z, c } = self;
        let num_iters = constants.num_iterations as u32;
        let mut i = 0;
        let mut prev_norm_sq = 0.0;
        let mut norm_sq = z.norm_squared();
        while norm_sq < 4.0 && i < num_iters {
            z = z * z + c;
            prev_norm_sq = norm_sq;
            norm_sq = z.norm_squared();
            i += 1;
            f(z);
        }

        let h = get_lerp_factor(prev_norm_sq, norm_sq);
        let inside = norm_sq < 4.0 || i == num_iters && constants.num_iterations.fract() < h;
        MandelbrotResult { inside, i, h }
    }
}

struct PerturbedMandelbrot<'a> {
    z0: Complex,
    dz: Complex,
    dc: Complex,
    reference_points: &'a [Complex],
    num_ref_iterations: usize,
}

impl Mandelbrot for PerturbedMandelbrot<'_> {
    fn z0(&self) -> Complex {
        self.z0
    }

    fn iterate<F: FnMut(Complex)>(
        self,
        constants: &FragmentConstants,
        mut f: F,
    ) -> MandelbrotResult {
        let PerturbedMandelbrot {
            z0,
            mut dz,
            dc,
            reference_points,
            num_ref_iterations,
        } = self;
        let num_iters = constants.num_iterations as u32;
        let mut i = 0;
        let mut prev_norm_sq = 0.0;
        let mut norm_sq = z0.norm_squared();
        let mut ref_i = 0;

        while norm_sq < 4.0 && i < num_iters {
            dz = 2.0 * reference_points[ref_i] * dz + dz * dz + dc;
            ref_i += 1;
            let z = reference_points[ref_i] + dz;
            prev_norm_sq = norm_sq;
            norm_sq = z.norm_squared();
            i += 1;
            f(z);
            if norm_sq < dz.norm_squared() || ref_i >= num_ref_iterations {
                dz = z;
                ref_i = 0;
            }
        }

        let h = get_lerp_factor(prev_norm_sq, norm_sq);
        let inside = norm_sq < 4.0 || i == num_iters && constants.num_iterations.fract() < h;
        MandelbrotResult { inside, i, h }
    }
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[cfg(not(feature = "emulate_constants"))]
    #[spirv(push_constant)]
    constants: &FragmentConstants,
    #[cfg(feature = "emulate_constants")]
    #[spirv(storage_buffer, descriptor_set = 3, binding = 0)]
    constants: &FragmentConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] iteration_points: &[Vec2],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)]
    mandelbrot_reference_points: &[Complex],
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] grid: &mut [RenderParameters],
    output: &mut Vec4,
) {
    let coord = frag_coord.xy();
    let size = constants.size.as_vec2();
    let is_split_vertical = size.x > size.y;
    let n = if is_split_vertical { Vec2::X } else { Vec2::Y };
    let render_julia_set = constants.render_julia_set.into();
    let mandelbrot_zoom = constants.mandelbrot_camera_zoom;
    let mandelbrot_uv =
        (coord - 0.5 * size) / size.y / mandelbrot_zoom + constants.mandelbrot_camera_translate;
    let is_julia = render_julia_set && coord.dot(n) > size.dot(n) * constants.render_split;

    let render_parameters = if constants.needs_reiterate_mandelbrot.into() && !is_julia {
        let dc = ((coord - 0.5 * size) / size.y / mandelbrot_zoom).into();
        let mandelbrot_input = PerturbedMandelbrot {
            z0: Complex::ZERO,
            dz: Complex::ZERO,
            dc,
            reference_points: mandelbrot_reference_points,
            num_ref_iterations: constants.mandelbrot_num_ref_iterations as usize,
        };
        let render_parameter_builder = RenderParameterBuilder {
            constants,
            mandelbrot_input,
        };
        let render_parameters = match constants.render_style {
            RenderStyle::Iterations => render_parameter_builder.iterations(),
            RenderStyle::Arg => render_parameter_builder.arg(),
            RenderStyle::LastDistance => render_parameter_builder.last_distance(),
        };

        let mut cell_grid = GridRefMut::new(GRID_SIZE, grid);
        cell_grid.set(coord.as_uvec2(), render_parameters);
        render_parameters
    } else if constants.needs_reiterate_julia.into() && is_julia {
        let z0 = ((coord - 0.5 * size) / size.y / constants.julia_camera_zoom
            + constants.julia_camera_translate)
            .into();
        let c: Complex = constants.marker.into();
        let mandelbrot_input = RegularMandelbrot { z0, c };
        let render_parameter_builder = RenderParameterBuilder {
            constants,
            mandelbrot_input,
        };
        let render_parameters = match constants.render_style {
            RenderStyle::Iterations => render_parameter_builder.iterations(),
            RenderStyle::Arg => render_parameter_builder.arg(),
            RenderStyle::LastDistance => render_parameter_builder.last_distance(),
        };
        let mut cell_grid = GridRefMut::new(GRID_SIZE, grid);
        cell_grid.set(coord.as_uvec2(), render_parameters);
        render_parameters
    } else {
        let cell_grid = GridRef::new(GRID_SIZE, grid);
        cell_grid.get(coord.as_uvec2())
    };
    let mut col = col_from_render_parameters(constants, render_parameters);

    // Slider
    if render_julia_set {
        let d = (coord + 0.5 - size * constants.render_split * n)
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
            let d = sdf::disk(coord - constants.marker_screen_space, MARKER_RADIUS);
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

fn col_from_render_parameters(
    constants: &FragmentConstants,
    render_parameters: RenderParameters,
) -> Vec3 {
    if render_parameters.inside.into() {
        return Vec3::ZERO;
    }
    let period = constants.palette_period;
    let t = constants.animate_time;
    let (period, t) = match constants.render_style {
        RenderStyle::Iterations => (0.2 * period, -t),
        RenderStyle::Arg => (
            (1 << (constants.palette_period * 3.0) as u32) as f32 / TAU,
            -t,
        ),
        RenderStyle::LastDistance => (period, t),
    };
    let x0 = render_parameters.x0 * period + t;
    let x1 = render_parameters.x1 * period + t;
    let col0 = get_col(constants.palette, x0);
    let col1 = get_col(constants.palette, x1);
    let s = smoothstep(0.0, constants.smooth_factor, render_parameters.h);
    col0.lerp(col1, s)
}

struct RenderParameterBuilder<'a, T> {
    constants: &'a FragmentConstants,
    mandelbrot_input: T,
}

impl<T: Mandelbrot> RenderParameterBuilder<'_, T> {
    fn iterations(self) -> RenderParameters {
        let mandelbrot = self.mandelbrot_input.iterate(self.constants, |_| {});
        if mandelbrot.inside {
            RenderParameters::new_inside()
        } else {
            let x0 = mandelbrot.i as f32;
            let x1 = (mandelbrot.i + 1) as f32;
            RenderParameters::new_outside(x0, x1, mandelbrot.h)
        }
    }

    fn arg(self) -> RenderParameters {
        let mut zs = [Complex::ZERO, self.mandelbrot_input.z0()];
        let mandelbrot = self.mandelbrot_input.iterate(self.constants, |z| {
            zs[0] = zs[1];
            zs[1] = z;
        });
        if mandelbrot.inside {
            RenderParameters::new_inside()
        } else {
            RenderParameters::new_outside(zs[0].arg(), zs[1].arg(), mandelbrot.h)
        }
    }

    fn last_distance(self) -> RenderParameters {
        let mut zs = [Complex::ZERO, Complex::ZERO, self.mandelbrot_input.z0()];
        let mandelbrot = self.mandelbrot_input.iterate(self.constants, |z| {
            zs[0] = zs[1];
            zs[1] = zs[2];
            zs[2] = z;
        });
        if mandelbrot.inside {
            RenderParameters::new_inside()
        } else {
            let ds0 = zs[0].distance(zs[1].0);
            let ds1 = zs[1].distance(zs[2].0);
            RenderParameters::new_outside(ds0, ds1, mandelbrot.h)
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
