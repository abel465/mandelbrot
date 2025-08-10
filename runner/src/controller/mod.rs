use crate::Options;
use crate::big_vec2::BigVec2;
use dashu::float::FBig;
use dashu::integer::IBig;
use easy_shader_runner::{ControllerTrait, GraphicsContext, UiState, egui, wgpu, winit};
use glam::*;
use shared::push_constants::shader::*;
use shared::*;
use std::collections::HashMap;
use std::str::FromStr;
use touch::*;
use web_time::Instant;
use winit::event::{ElementState, MouseButton, TouchPhase};

mod keyboard;
mod touch;
mod ui;

const MAX_ZOOM_MANDELBROT: f64 = 1e36;
const MAX_ZOOM_JULIA: f64 = 999999.9;
const MAX_ITER_POINTS: u32 = 1307;
const MAX_ADDITIONAL_ITERS: u32 = 200;
const PRECISION: usize = 192;

struct Cameras {
    mandelbrot: Camera,
    julia: Camera,
}

impl Cameras {
    fn iter_mut(&mut self) -> [&mut Camera; 2] {
        [&mut self.mandelbrot, &mut self.julia]
    }
}

impl Default for Cameras {
    fn default() -> Self {
        Self {
            julia: Camera::new(0.25, BigVec2::from_f64s(0.0, 0.0).with_precision(PRECISION)),
            mandelbrot: Camera::new(
                0.3,
                BigVec2::from_f64s(-0.75, 0.0).with_precision(PRECISION),
            ),
        }
    }
}

struct Camera {
    zoom: f64,
    translate: BigVec2,
    grabbing: bool,
    needs_reiterate: bool,
}

impl Camera {
    fn new(zoom: f64, translate: BigVec2) -> Self {
        Self {
            zoom,
            translate,
            grabbing: false,
            needs_reiterate: true,
        }
    }

    #[allow(dead_code)]
    fn deep_mandelbrot() -> Self {
        Self::new(
            3e35,
            BigVec2::new(
                FBig::from_parts(
                    IBig::from_str("-186072271889131572891447674874887788791").unwrap(),
                    -129,
                ),
                FBig::from_parts(
                    IBig::from_str("143326526706623085196071470634592994379").unwrap(),
                    -127,
                ),
            )
            .with_precision(PRECISION),
        )
    }
}

#[derive(Clone, Copy, Default)]
struct MarkerIterationStats {
    final_norm: f32,
    final_distance: f32,
    final_angle: f32,
    angle_sum: f32,
    distance_sum: f32,
    norm_sum: f32,
    proximity: f32, // normalized proximity to the next iteration
    count: u32,
}

struct MarkerIterations {
    enabled: bool,
    dragging: bool,
    position: BigVec2,
    points: Vec<Vec2>,
    points_buffer: Option<wgpu::Buffer>,
    recompute: bool,
    stats: MarkerIterationStats,
}

impl Default for MarkerIterations {
    fn default() -> Self {
        MarkerIterations {
            enabled: false,
            dragging: false,
            position: BigVec2::from_f64s(-0.767294, -0.169140),
            points: vec![],
            points_buffer: None,
            recompute: false,
            stats: MarkerIterationStats::default(),
        }
    }
}

struct RenderSplit {
    value: f64,
    dragging: Option<egui::CursorIcon>,
}

impl Default for RenderSplit {
    fn default() -> Self {
        Self {
            value: 0.5,
            dragging: None,
        }
    }
}

struct Smooth {
    enable: bool,
    value: f32,
}

impl Default for Smooth {
    fn default() -> Self {
        Self {
            enable: true,
            value: 1.0,
        }
    }
}

impl Smooth {
    fn factor(&self) -> f32 {
        if self.enable { self.value } else { 0.0 }
    }
}

struct Animate {
    enable: bool,
    value: f32,
    last_instant: Instant,
    speed: f32,
    reverse: bool,
}

impl Default for Animate {
    fn default() -> Self {
        Self {
            enable: true,
            value: 0.0,
            last_instant: Instant::now(),
            speed: 0.1,
            reverse: false,
        }
    }
}

impl Animate {
    fn tick(&mut self) {
        if self.enable {
            let sgn = if self.reverse { -1.0 } else { 1.0 };
            self.value += sgn * self.speed * self.last_instant.elapsed().as_secs_f32();
        }
        self.last_instant = Instant::now();
    }
}

struct MandelbrotReference {
    buffer: Option<wgpu::Buffer>,
    points: Vec<Vec2>,
    num_ref_iterations: u32,
    recompute: bool,
}

impl Default for MandelbrotReference {
    fn default() -> Self {
        Self {
            buffer: None,
            points: vec![],
            num_ref_iterations: 0,
            recompute: true,
        }
    }
}

#[derive(Default)]
struct DeltaParams {
    iterations: f64,
    period: f64,
    zoom: f64,
    translate: DVec2,
    animation_speed: f64,
    exponent: f64,
}

struct NumIterations {
    pub n: f64,
    mode: NumIterationsMode,
}

enum NumIterationsMode {
    Additional,
    Fixed,
}

impl NumIterations {
    fn calculate_additional_iterations(&self, zoom: f64) -> f64 {
        (zoom + 1.0).log2() * 9.0
    }

    fn calculate_num_iterations(&self, zoom: f64) -> f64 {
        match self.mode {
            NumIterationsMode::Additional => {
                (self.calculate_additional_iterations(zoom) + self.n).max(1.0)
            }
            NumIterationsMode::Fixed => self.n,
        }
    }

    fn toggle_mode(&mut self, zoom: f64) {
        self.mode = match self.mode {
            NumIterationsMode::Additional => {
                self.n = self.calculate_num_iterations(zoom);
                NumIterationsMode::Fixed
            }
            NumIterationsMode::Fixed => {
                self.n = self.n - self.calculate_additional_iterations(zoom);
                NumIterationsMode::Additional
            }
        };
    }

    fn slider_range(&self, zoom: f64) -> std::ops::RangeInclusive<f64> {
        match self.mode {
            NumIterationsMode::Additional => {
                -self.calculate_additional_iterations(zoom)..=MAX_ADDITIONAL_ITERS as f64
            }
            NumIterationsMode::Fixed => 0.0..=(MAX_ITER_POINTS - 1) as f64,
        }
    }

    fn prev_whole_iteration(&mut self, zoom: f64) {
        match self.mode {
            NumIterationsMode::Additional => {
                let n = self.calculate_additional_iterations(zoom);
                self.n = (n + self.n).ceil() - 1.0 - n;
            }
            NumIterationsMode::Fixed => {
                self.n = self.n.ceil() - 1.0;
            }
        };
    }

    fn next_whole_iteration(&mut self, zoom: f64) {
        match self.mode {
            NumIterationsMode::Additional => {
                let n = self.calculate_additional_iterations(zoom);
                self.n = (n + self.n).floor() + 1.0 - n;
            }
            NumIterationsMode::Fixed => {
                self.n = self.n.floor() + 1.0;
            }
        };
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct WasmStuff {
    ui_rects: Vec<egui::Rect>,
    pixels_per_point: f32,
}

pub struct Controller {
    size: UVec2,
    start: Instant,
    last_instant: Instant,
    cursor: DVec2,
    mouse_button_pressed: u32,
    cameras: Cameras,
    debug: bool,
    num_iterations: NumIterations,
    marker_iterations: MarkerIterations,
    context_menu: Option<DVec2>,
    render_julia_set: bool,
    render_split: RenderSplit,
    palette: Palette,
    palette_period: f32,
    smooth: Smooth,
    animate: Animate,
    show_fps: bool,
    render_style: RenderStyle,
    mandelbrot_reference: MandelbrotReference,
    render_partitioning: RenderPartitioning,
    delta_params: DeltaParams,
    exponent: f64,
    escape_radius: f32,
    iteration_mode: IterationMode,
    ctrl_down: bool,
    touches: HashMap<u64, Touch>,
    #[cfg(target_arch = "wasm32")]
    wasm_stuff: WasmStuff,
}

impl Controller {
    pub fn new(options: &Options) -> Self {
        debug_assert!(
            MAX_ITER_POINTS
                >= NumIterations {
                    n: MAX_ITER_POINTS as f64,
                    mode: NumIterationsMode::Fixed
                }
                .calculate_num_iterations(MAX_ZOOM_MANDELBROT) as u32
        );
        debug_assert!(
            MAX_ITER_POINTS
                >= NumIterations {
                    n: MAX_ADDITIONAL_ITERS as f64,
                    mode: NumIterationsMode::Additional
                }
                .calculate_num_iterations(MAX_ZOOM_MANDELBROT) as u32
        );
        let cameras = Cameras::default();
        Self {
            size: UVec2::ZERO,
            start: Instant::now(),
            last_instant: Instant::now(),
            cursor: DVec2::ZERO,
            mouse_button_pressed: 0,
            num_iterations: NumIterations {
                n: 25.0,
                mode: NumIterationsMode::Additional,
            },
            cameras,
            debug: options.debug,
            marker_iterations: MarkerIterations::default(),
            context_menu: None,
            render_julia_set: false,
            render_split: RenderSplit::default(),
            palette: Palette::default(),
            palette_period: 0.5,
            smooth: Smooth::default(),
            animate: Animate::default(),
            show_fps: false,
            render_style: RenderStyle::default(),
            mandelbrot_reference: MandelbrotReference::default(),
            render_partitioning: RenderPartitioning::default(),
            delta_params: DeltaParams::default(),
            exponent: 2.0,
            escape_radius: 2.0,
            iteration_mode: IterationMode::default(),
            ctrl_down: false,
            touches: HashMap::new(),
            #[cfg(target_arch = "wasm32")]
            wasm_stuff: WasmStuff::default(),
        }
    }

    fn to_uv_space_big(&self, p: DVec2) -> BigVec2 {
        let size = self.size.as_dvec2();
        self.cameras.mandelbrot.translate.clone()
            + BigVec2::from_dvec2(
                (p - 0.5 * size) * dvec2(size.x / size.y, 1.0)
                    / self.cameras.mandelbrot.zoom
                    / size,
            )
    }

    fn to_screen_space_big(&self, p: &BigVec2) -> DVec2 {
        let size = self.size.as_dvec2();
        (p.clone() - self.cameras.mandelbrot.translate.clone()).as_dvec2()
            / dvec2(size.x / size.y, 1.0)
            * self.cameras.mandelbrot.zoom
            * size
            + 0.5 * size
    }

    fn pos_on_render_split(&self, pos: DVec2) -> Option<egui::CursorIcon> {
        let (n, icon) = if self.size.x > self.size.y {
            (DVec2::X, egui::CursorIcon::ResizeHorizontal)
        } else {
            (DVec2::Y, egui::CursorIcon::ResizeVertical)
        };
        (self.render_julia_set
            && ((pos / self.size.as_dvec2()).dot(n).abs() - self.render_split.value).abs() < 0.004)
            .then_some(icon)
    }

    fn can_grab_render_split(&self) -> Option<egui::CursorIcon> {
        self.pos_on_render_split(self.cursor)
    }

    fn can_grab_marker(&self) -> bool {
        self.pos_on_marker(self.cursor)
    }

    fn pos_on_marker(&self, pos: DVec2) -> bool {
        (self.marker_iterations.enabled || self.render_julia_set)
            && !self.is_cursor_in_julia()
            && pos.distance_squared(self.to_screen_space_big(&self.marker_iterations.position))
                < MARKER_RADIUS as f64 * MARKER_RADIUS as f64
    }

    fn is_pos_in_julia(&self, pos: DVec2) -> bool {
        let size = self.size.as_dvec2();
        let is_split_vertical = size.x > size.y;
        let n = if is_split_vertical {
            DVec2::X
        } else {
            DVec2::Y
        };
        self.render_julia_set && pos.dot(n) > size.dot(n) * self.render_split.value
    }

    fn is_cursor_in_julia(&self) -> bool {
        self.is_pos_in_julia(self.cursor)
    }

    fn camera(&mut self) -> &mut Camera {
        self.camera_from_pos(self.cursor)
    }

    fn camera_from_pos(&mut self, pos: DVec2) -> &mut Camera {
        if self.is_pos_in_julia(pos) {
            &mut self.cameras.julia
        } else {
            &mut self.cameras.mandelbrot
        }
    }

    fn max_zoom(&self) -> f64 {
        self.max_zoom_from_pos(self.cursor)
    }

    fn max_zoom_from_pos(&self, pos: DVec2) -> f64 {
        if self.is_pos_in_julia(pos) {
            MAX_ZOOM_JULIA
        } else {
            MAX_ZOOM_MANDELBROT
        }
    }

    fn calculate_num_iterations(&self) -> f64 {
        self.num_iterations
            .calculate_num_iterations(self.cameras.mandelbrot.zoom)
    }
}

impl ControllerTrait for Controller {
    fn resize(&mut self, size: UVec2) {
        self.size = size;
        self.cameras.mandelbrot.needs_reiterate = true;
        self.cameras.julia.needs_reiterate = true;
    }

    fn keyboard_input(&mut self, key: winit::event::KeyEvent) {
        self.keyboard_input_impl(key);
    }

    fn mouse_move(&mut self, position: DVec2) {
        let prev_cursor = self.cursor;
        self.cursor = position;
        if self.marker_iterations.dragging {
            self.marker_iterations.position +=
                self.to_uv_space_big(self.cursor) - self.to_uv_space_big(prev_cursor);
            self.marker_iterations.recompute = self.marker_iterations.enabled;
            self.cameras.julia.needs_reiterate = true;
        } else if self.render_split.dragging.is_some() {
            let size = self.size.as_dvec2();
            let delta = (prev_cursor - self.cursor) / size;
            let value = if size.x > size.y { delta.x } else { delta.y };
            self.render_split.value -= value;
            if value > 0.0 {
                self.cameras.julia.needs_reiterate = true;
            } else if value < 0.0 {
                self.cameras.mandelbrot.needs_reiterate = true;
            }
        } else {
            if self.cameras.mandelbrot.grabbing {
                self.mandelbrot_reference.recompute = true;
            }
            for camera in self.cameras.iter_mut() {
                if camera.grabbing {
                    self.context_menu = None;
                    let delta =
                        BigVec2::from_dvec2((prev_cursor - self.cursor) / self.size.y as f64)
                            .with_precision(PRECISION);
                    camera.translate += delta / camera.zoom;
                    camera.needs_reiterate = true;
                }
            }
        }
    }

    fn touch(&mut self, id: u64, phase: TouchPhase, position: DVec2) {
        self.touch_impl(id, phase, position);
    }

    fn mouse_scroll(&mut self, delta: DVec2) {
        if delta.y == 0.0 {
            return;
        }
        let cursor = self.cursor;
        let size = self.size.as_dvec2();
        let max_zoom = self.max_zoom();
        let camera = self.camera();
        let val = delta.y * 0.1;
        let prev_zoom = camera.zoom;
        let mouse_pos0 = BigVec2::from_dvec2(cursor - size / 2.0) / camera.zoom / size.y;
        camera.zoom = (prev_zoom * (1.0 + val)).clamp(0.05, max_zoom);
        let mouse_pos1 = BigVec2::from_dvec2(cursor - size / 2.0) / camera.zoom / size.y;
        camera.translate += mouse_pos0 - mouse_pos1;
        self.cameras.julia.needs_reiterate = true;
        if !self.is_cursor_in_julia() {
            self.mandelbrot_reference.recompute = true;
            self.cameras.mandelbrot.needs_reiterate = true;
            self.marker_iterations.recompute = self.marker_iterations.enabled;
        }
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        let mask = 1
            << match button {
                MouseButton::Left => {
                    if matches!(state, ElementState::Pressed) {
                        self.marker_iterations.dragging = self.can_grab_marker();
                        self.render_split.dragging = self.can_grab_render_split();
                        self.camera().grabbing = true;
                    } else {
                        self.marker_iterations.dragging = false;
                        self.render_split.dragging = None;
                        for camera in self.cameras.iter_mut() {
                            camera.grabbing = false;
                        }
                        self.render_split.value = self.render_split.value.clamp(0.0, 1.0);
                    }
                    0
                }
                MouseButton::Middle => 1,
                MouseButton::Right => {
                    if matches!(state, ElementState::Pressed) {
                        self.context_menu = Some(self.cursor);
                    }
                    2
                }
                MouseButton::Back => 3,
                MouseButton::Forward => 4,
                MouseButton::Other(i) => 5 + (i as usize),
            };
        match state {
            ElementState::Pressed => self.mouse_button_pressed |= mask,
            ElementState::Released => self.mouse_button_pressed &= !mask,
        }
    }

    fn prepare_render(
        &mut self,
        _gfx_ctx: &GraphicsContext,
        _offset: Vec2,
    ) -> impl bytemuck::NoUninit {
        self.animate.tick();
        let needs_reiterate_mandelbrot = self.cameras.mandelbrot.needs_reiterate;
        let needs_reiterate_julia = self.cameras.julia.needs_reiterate;
        self.cameras.mandelbrot.needs_reiterate = false;
        self.cameras.julia.needs_reiterate = false;
        FragmentConstants {
            size: self.size.into(),
            time: self.start.elapsed().as_secs_f32(),
            mandelbrot_camera_translate: self.cameras.mandelbrot.translate.as_vec2(),
            mandelbrot_camera_zoom: self.cameras.mandelbrot.zoom as f32,
            julia_camera_translate: self.cameras.julia.translate.as_vec2(),
            julia_camera_zoom: self.cameras.julia.zoom as f32,
            num_iterations: self.calculate_num_iterations() as f32,
            show_iterations: (self.marker_iterations.enabled
                && !self.marker_iterations.points.is_empty())
            .into(),
            num_points: self.marker_iterations.points.len() as u32,
            marker: self.marker_iterations.position.as_vec2(),
            marker_screen_space: self
                .to_screen_space_big(&self.marker_iterations.position)
                .as_vec2(),
            render_julia_set: self.render_julia_set.into(),
            render_split: self.render_split.value as f32,
            palette: self.palette,
            smooth_factor: self.smooth.factor(),
            animate_time: self.animate.value,
            palette_period: self.palette_period,
            render_style: self.render_style,
            mandelbrot_num_ref_iterations: self.mandelbrot_reference.num_ref_iterations,
            needs_reiterate_mandelbrot: needs_reiterate_mandelbrot.into(),
            needs_reiterate_julia: needs_reiterate_julia.into(),
            iteration_mode: self.iteration_mode,
            render_partitioning: self.render_partitioning,
            exponent: self.exponent as f32,
            escape_radius: self.escape_radius,
        }
    }

    fn describe_bind_groups(
        &mut self,
        gfx_ctx: &GraphicsContext,
    ) -> (Vec<wgpu::BindGroupLayout>, Vec<wgpu::BindGroup>) {
        let device = &gfx_ctx.device;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("bind_group_layout"),
        });

        use wgpu::util::DeviceExt;
        let marker_iteration_points_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("inside_particles_buffer"),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                contents: &[0; std::mem::size_of::<Vec2>() * MAX_ITER_POINTS as usize],
            });
        let perturbation_points_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("inside_particles_buffer"),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                contents: &[0; std::mem::size_of::<Vec2>() * MAX_ITER_POINTS as usize],
            });
        let render_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fading_particles_buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            contents: &[0; std::mem::size_of::<RenderParameters>()
                * GRID_SIZE.x as usize
                * GRID_SIZE.y as usize],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: marker_iteration_points_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: perturbation_points_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: render_params_buffer.as_entire_binding(),
                },
            ],
            label: Some("particles_bind_group"),
        });

        self.marker_iterations.points_buffer = Some(marker_iteration_points_buffer);
        self.mandelbrot_reference.buffer = Some(perturbation_points_buffer);

        (vec![layout], vec![bind_group])
    }

    fn ui(&mut self, ctx: &egui::Context, ui_state: &mut UiState, gfx_ctx: &GraphicsContext) {
        self.ui_impl(ctx, ui_state, gfx_ctx);
    }
}
