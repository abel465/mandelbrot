use crate::big_vec2::BigVec2;
use crate::Options;
use dashu::float::FBig;
use dashu::integer::IBig;
use easy_shader_runner::{egui, wgpu, winit, ControllerTrait, UiState};
use glam::*;
use shared::push_constants::shader::*;
use shared::*;
use std::str::FromStr;
use web_time::Instant;
use winit::event::{ElementState, MouseButton};
use winit::{event::KeyEvent, keyboard::Key};

mod ui;

const MAX_ZOOM: f64 = 1e36;
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
struct IterationStats {
    final_norm: f32,
    final_distance: f32,
    final_angle: f32,
    angle_sum: f32,
    distance_sum: f32,
    norm_sum: f32,
    proximity: f32, // normalized proximity to the next iteration
    count: u32,
}

struct Iterations {
    enabled: bool,
    dragging: bool,
    marker: BigVec2,
    points: Vec<Vec2>,
    points_buffer: Option<wgpu::Buffer>,
    recompute: bool,
    stats: IterationStats,
    mode: IterationMode,
}

impl Default for Iterations {
    fn default() -> Self {
        Iterations {
            enabled: false,
            dragging: false,
            marker: BigVec2::from_f64s(-0.767294, -0.169140),
            points: vec![],
            points_buffer: None,
            recompute: false,
            stats: IterationStats::default(),
            mode: IterationMode::default(),
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
        if self.enable {
            self.value
        } else {
            0.0
        }
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

pub struct Controller {
    size: UVec2,
    start: Instant,
    last_instant: Instant,
    cursor: DVec2,
    mouse_button_pressed: u32,
    cameras: Cameras,
    debug: bool,
    num_iterations: f64,
    iterations: Iterations,
    context_menu: Option<DVec2>,
    render_julia_set: bool,
    render_split: RenderSplit,
    palette: Palette,
    palette_period: f32,
    smooth: Smooth,
    animate: Animate,
    show_fps: bool,
    render_style: RenderStyle,
    additional_iterations: f64,
    mandelbrot_reference: MandelbrotReference,
    render_partitioning: RenderPartitioning,
    delta_params: DeltaParams,
    exponent: f64,
    escape_radius: f32,
}

impl Controller {
    pub fn new(options: &Options) -> Self {
        debug_assert!(
            MAX_ITER_POINTS
                >= calculate_num_iterations(MAX_ZOOM, MAX_ADDITIONAL_ITERS as f64) as u32
        );
        let additional_iterations = 25.0;
        let cameras = Cameras::default();
        Self {
            size: UVec2::ZERO,
            start: Instant::now(),
            last_instant: Instant::now(),
            cursor: DVec2::ZERO,
            mouse_button_pressed: 0,
            num_iterations: calculate_num_iterations(
                cameras.mandelbrot.zoom,
                additional_iterations,
            ),
            cameras,
            debug: options.debug,
            iterations: Iterations::default(),
            context_menu: None,
            render_julia_set: false,
            render_split: RenderSplit::default(),
            palette: Palette::default(),
            palette_period: 0.5,
            smooth: Smooth::default(),
            animate: Animate::default(),
            show_fps: false,
            render_style: RenderStyle::default(),
            additional_iterations,
            mandelbrot_reference: MandelbrotReference::default(),
            render_partitioning: RenderPartitioning::default(),
            delta_params: DeltaParams::default(),
            exponent: 2.0,
            escape_radius: 2.0,
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

    fn can_grab_render_split(&self) -> Option<egui::CursorIcon> {
        let (n, icon) = if self.size.x > self.size.y {
            (DVec2::X, egui::CursorIcon::ResizeHorizontal)
        } else {
            (DVec2::Y, egui::CursorIcon::ResizeVertical)
        };
        (self.render_julia_set
            && ((self.cursor / self.size.as_dvec2()).dot(n).abs() - self.render_split.value).abs()
                < 0.004)
            .then_some(icon)
    }

    fn can_grab_marker(&self) -> bool {
        (self.iterations.enabled || self.render_julia_set)
            && !self.is_cursor_in_julia()
            && self
                .cursor
                .distance_squared(self.to_screen_space_big(&self.iterations.marker))
                < MARKER_RADIUS as f64 * MARKER_RADIUS as f64
    }

    fn is_cursor_in_julia(&self) -> bool {
        let size = self.size.as_dvec2();
        let cursor = self.cursor;
        let is_split_vertical = size.x > size.y;
        let n = if is_split_vertical {
            DVec2::X
        } else {
            DVec2::Y
        };
        self.render_julia_set && cursor.dot(n) > size.dot(n) * self.render_split.value
    }

    fn camera(&mut self) -> &mut Camera {
        if self.is_cursor_in_julia() {
            &mut self.cameras.julia
        } else {
            &mut self.cameras.mandelbrot
        }
    }

    fn max_zoom(&self) -> f64 {
        if self.is_cursor_in_julia() {
            999999.9
        } else {
            MAX_ZOOM
        }
    }
}

impl ControllerTrait for Controller {
    fn resize(&mut self, size: UVec2) {
        self.size = size;
        self.cameras.mandelbrot.needs_reiterate = true;
        self.cameras.julia.needs_reiterate = true;
    }

    fn keyboard_input(&mut self, key: KeyEvent) {
        if !key.state.is_pressed() {
            match key.logical_key {
                Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
                    self.delta_params.translate.y = self.delta_params.translate.y.min(0.0);
                }
                Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                    self.delta_params.translate.y = self.delta_params.translate.y.max(0.0);
                }
                Key::Named(winit::keyboard::NamedKey::ArrowLeft) => {
                    self.delta_params.translate.x = self.delta_params.translate.x.max(0.0);
                }
                Key::Named(winit::keyboard::NamedKey::ArrowRight) => {
                    self.delta_params.translate.x = self.delta_params.translate.x.min(0.0);
                }
                Key::Character(c) => {
                    let c = c.chars().next().unwrap();
                    match c {
                        'z' => {
                            if self.delta_params.zoom > 1.0 {
                                self.delta_params.zoom = 0.0;
                            }
                        }
                        'x' => {
                            if self.delta_params.zoom < 1.0 {
                                self.delta_params.zoom = 0.0;
                            }
                        }
                        'p' => {
                            if self.delta_params.period > 1.0 {
                                self.delta_params.period = 0.0;
                            }
                        }
                        'o' => {
                            if self.delta_params.period < 1.0 {
                                self.delta_params.period = 0.0;
                            }
                        }
                        'j' => {
                            if self.delta_params.animation_speed < 1.0 {
                                self.delta_params.animation_speed = 0.0;
                            }
                        }
                        'l' => {
                            if self.delta_params.animation_speed > 1.0 {
                                self.delta_params.animation_speed = 0.0;
                            }
                        }
                        'u' => {
                            self.delta_params.iterations = self.delta_params.iterations.max(0.0);
                        }
                        'i' => {
                            self.delta_params.iterations = self.delta_params.iterations.min(0.0);
                        }
                        'g' => {
                            self.delta_params.exponent = self.delta_params.exponent.max(0.0);
                        }
                        'h' => {
                            self.delta_params.exponent = self.delta_params.exponent.min(0.0);
                        }
                        'G' => {
                            self.exponent = self.exponent.ceil() - 1.0;
                            self.iterations.recompute = self.iterations.enabled;
                            self.mandelbrot_reference.recompute = true;
                            self.cameras.mandelbrot.needs_reiterate = true;
                            self.cameras.julia.needs_reiterate = true;
                        }
                        'H' => {
                            self.exponent = self.exponent.floor() + 1.0;
                            self.iterations.recompute = self.iterations.enabled;
                            self.mandelbrot_reference.recompute = true;
                            self.cameras.mandelbrot.needs_reiterate = true;
                            self.cameras.julia.needs_reiterate = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            return;
        }
        let move_speed = 0.2;
        match key.logical_key {
            Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
                self.delta_params.translate.y = move_speed;
            }
            Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                self.delta_params.translate.y = -move_speed;
            }
            Key::Named(winit::keyboard::NamedKey::ArrowLeft) => {
                self.delta_params.translate.x = -move_speed;
            }
            Key::Named(winit::keyboard::NamedKey::ArrowRight) => {
                self.delta_params.translate.x = move_speed;
            }
            Key::Character(c) => {
                let c = c.chars().next().unwrap();
                match c {
                    'z' | 'x' => {
                        let z = 1.4;
                        self.delta_params.zoom = match c {
                            'z' => z,
                            'x' => 1.0 / z,
                            _ => unreachable!(),
                        };
                    }
                    'p' | 'o' => {
                        let z = 1.2;
                        self.delta_params.period = match c {
                            'p' => z,
                            'o' => 1.0 / z,
                            _ => unreachable!(),
                        };
                    }
                    'k' => {
                        self.animate.enable = !self.animate.enable;
                    }
                    'j' | 'l' => {
                        let z = 2.0;
                        self.delta_params.animation_speed = match c {
                            'l' => z,
                            'j' => 1.0 / z,
                            _ => unreachable!(),
                        };
                    }
                    'u' | 'i' => {
                        let z = 5.0;
                        self.delta_params.iterations = match c {
                            'u' => -z,
                            'i' => z,
                            _ => unreachable!(),
                        };
                    }
                    'g' | 'h' => {
                        let z = 0.2;
                        self.delta_params.exponent = match c {
                            'g' => -z,
                            'h' => z,
                            _ => unreachable!(),
                        };
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn mouse_move(&mut self, position: DVec2) {
        let prev_cursor = self.cursor;
        self.cursor = position;
        if self.iterations.dragging {
            self.iterations.marker +=
                self.to_uv_space_big(self.cursor) - self.to_uv_space_big(prev_cursor);
            self.iterations.recompute = self.iterations.enabled;
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
        self.num_iterations = calculate_num_iterations(
            self.cameras.mandelbrot.zoom,
            self.additional_iterations as f64,
        );
        self.cameras.julia.needs_reiterate = true;
        if !self.is_cursor_in_julia() {
            self.mandelbrot_reference.recompute = true;
            self.cameras.mandelbrot.needs_reiterate = true;
            self.iterations.recompute = self.iterations.enabled;
        }
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        let mask = 1
            << match button {
                MouseButton::Left => {
                    if matches!(state, ElementState::Pressed) {
                        self.iterations.dragging = self.can_grab_marker();
                        self.render_split.dragging = self.can_grab_render_split();
                        self.camera().grabbing = true;
                    } else {
                        self.iterations.dragging = false;
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

    fn prepare_render(&mut self, _offset: Vec2) -> impl bytemuck::NoUninit {
        self.animate.tick();
        let needs_reiterate_mandelbrot = self.cameras.mandelbrot.needs_reiterate;
        let needs_reiterate_julia = self.cameras.julia.needs_reiterate;
        self.cameras.mandelbrot.needs_reiterate = false;
        self.cameras.julia.needs_reiterate = false;
        FragmentConstants {
            size: self.size.into(),
            time: self.start.elapsed().as_secs_f32(),
            cursor: self.cursor.as_vec2(),
            mandelbrot_camera_translate: self.cameras.mandelbrot.translate.as_vec2(),
            mandelbrot_camera_zoom: self.cameras.mandelbrot.zoom as f32,
            julia_camera_translate: self.cameras.julia.translate.as_vec2(),
            julia_camera_zoom: self.cameras.julia.zoom as f32,
            num_iterations: self.num_iterations as f32,
            show_iterations: (self.iterations.enabled && !self.iterations.points.is_empty()).into(),
            num_points: self.iterations.points.len() as u32,
            marker: self.iterations.marker.as_vec2(),
            marker_screen_space: self.to_screen_space_big(&self.iterations.marker).as_vec2(),
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
            iteration_mode: self.iterations.mode,
            render_partitioning: self.render_partitioning,
            exponent: self.exponent as f32,
            escape_radius: self.escape_radius,
        }
    }

    fn describe_buffers(&self) -> Vec<easy_shader_runner::BufferDescriptor> {
        use easy_shader_runner::wgpu;
        vec![
            easy_shader_runner::BufferDescriptor {
                data: &[0; std::mem::size_of::<Vec2>() * MAX_ITER_POINTS as usize],
                read_only: true,
                shader_stages: wgpu::ShaderStages::FRAGMENT,
                cpu_writable: true,
            },
            easy_shader_runner::BufferDescriptor {
                data: &[0; std::mem::size_of::<Vec2>() * MAX_ITER_POINTS as usize],
                read_only: true,
                shader_stages: wgpu::ShaderStages::FRAGMENT,
                cpu_writable: true,
            },
            easy_shader_runner::BufferDescriptor {
                data: &[0; std::mem::size_of::<RenderParameters>()
                    * GRID_SIZE.x as usize
                    * GRID_SIZE.y as usize],
                read_only: false,
                shader_stages: wgpu::ShaderStages::FRAGMENT,
                cpu_writable: false,
            },
        ]
    }

    fn receive_buffers(&mut self, mut buffers: Vec<wgpu::Buffer>) {
        debug_assert!(buffers.len() == 2);
        self.mandelbrot_reference.buffer = Some(buffers.pop().unwrap());
        self.iterations.points_buffer = Some(buffers.pop().unwrap());
    }

    fn ui(
        &mut self,
        ctx: &egui::Context,
        ui_state: &mut UiState,
        graphics_context: &easy_shader_runner::GraphicsContext,
    ) {
        self.ui_impl(ctx, ui_state, graphics_context);
    }
}

fn calculate_num_iterations(zoom: f64, c: f64) -> f64 {
    ((zoom + 1.0).log2() * 9.0 + c).max(1.0)
}
