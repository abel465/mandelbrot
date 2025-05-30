use crate::Options;
use easy_shader_runner::{egui, wgpu, winit, ControllerTrait, UiState};
use glam::*;
use shared::push_constants::shader::*;
use shared::MARKER_RADIUS;
use web_time::Instant;
use winit::event::{ElementState, MouseButton};

mod ui;

const MAX_ZOOM: f32 = 999999.9;
const MAX_ITER_POINTS: u32 = 380;
const MAX_ADDITIONAL_ITERS: u32 = 200;

struct Cameras {
    has_calibrated: bool,
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
            has_calibrated: false,
            julia: Camera {
                zoom: 0.25,
                translate: vec2(-1.3, 0.0),
                grabbing: false,
            },
            mandelbrot: Camera {
                zoom: 0.25,
                translate: vec2(0.6, -0.3),
                grabbing: false,
            },
        }
    }
}

struct Camera {
    zoom: f32,
    translate: Vec2,
    grabbing: bool,
}

#[derive(Clone, Copy, Default)]
struct IterationStats {
    last_norm_sq: f32,
    last_distance: f32,
    last_angle: f32,
    total_angle: f32,
    total_distance: f32,
    proximity: f32, // normalized proximity to the next iteration
    count: u32,
}

struct Iterations {
    enabled: bool,
    dragging: bool,
    marker: Vec2,
    points: Vec<Vec2>,
    points_buffer: Option<wgpu::Buffer>,
    recompute: bool,
    stats: IterationStats,
}

impl Default for Iterations {
    fn default() -> Self {
        Iterations {
            enabled: true,
            dragging: false,
            marker: vec2(-0.767294, -0.169140),
            points: vec![],
            points_buffer: None,
            recompute: true,
            stats: IterationStats::default(),
        }
    }
}

struct RenderSplit {
    value: f32,
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
            value: 0.5,
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

pub struct Controller {
    size: UVec2,
    start: Instant,
    cursor: Vec2,
    prev_cursor: Vec2,
    mouse_button_pressed: u32,
    cameras: Cameras,
    debug: bool,
    num_iterations: f32,
    iterations: Iterations,
    context_menu: Option<Vec2>,
    render_julia_set: bool,
    render_split: RenderSplit,
    palette: Palette,
    palette_period: f32,
    smooth: Smooth,
    animate: Animate,
    show_fps: bool,
    render_style: RenderStyle,
    additional_iterations: u32,
}

impl Controller {
    pub fn new(options: &Options) -> Self {
        debug_assert!(
            MAX_ITER_POINTS
                >= calculate_num_iterations(MAX_ZOOM, MAX_ADDITIONAL_ITERS as f32) as u32
        );
        let additional_iterations = 25;
        let cameras = Cameras::default();
        Self {
            size: UVec2::ZERO,
            start: Instant::now(),
            cursor: Vec2::ZERO,
            prev_cursor: Vec2::ZERO,
            mouse_button_pressed: 0,
            num_iterations: calculate_num_iterations(
                cameras.mandelbrot.zoom,
                additional_iterations as f32,
            ),
            cameras,
            debug: options.debug,
            iterations: Iterations::default(),
            context_menu: None,
            render_julia_set: true,
            render_split: RenderSplit::default(),
            palette: Palette::default(),
            palette_period: 0.5,
            smooth: Smooth::default(),
            animate: Animate::default(),
            show_fps: false,
            render_style: RenderStyle::default(),
            additional_iterations,
        }
    }

    fn to_uv(&self, p: Vec2) -> Vec2 {
        let size = self.size.as_vec2();
        self.cameras.mandelbrot.translate
            + (p - 0.5 * size) * vec2(size.x / size.y, 1.0) / self.cameras.mandelbrot.zoom / size
    }

    fn from_uv(&self, p: Vec2) -> Vec2 {
        let size = self.size.as_vec2();
        (p - self.cameras.mandelbrot.translate) / vec2(size.x / size.y, 1.0)
            * self.cameras.mandelbrot.zoom
            * size
            + 0.5 * size
    }

    fn can_grab_render_split(&self) -> Option<egui::CursorIcon> {
        let (n, icon) = if self.size.x > self.size.y {
            (Vec2::X, egui::CursorIcon::ResizeHorizontal)
        } else {
            (Vec2::Y, egui::CursorIcon::ResizeVertical)
        };
        (self.render_julia_set
            && ((self.cursor / self.size.as_vec2()).dot(n).abs() - self.render_split.value).abs()
                < 0.004)
            .then_some(icon)
    }

    fn can_grab_marker(&self) -> bool {
        (self.iterations.enabled || self.render_julia_set)
            && !self.is_cursor_in_julia()
            && self
                .cursor
                .distance_squared(self.from_uv(self.iterations.marker))
                < MARKER_RADIUS * MARKER_RADIUS
    }

    fn is_cursor_in_julia(&self) -> bool {
        let size = self.size.as_vec2();
        let cursor = self.cursor;
        let is_split_vertical = size.x > size.y;
        let n = if is_split_vertical { Vec2::X } else { Vec2::Y };
        self.render_julia_set && cursor.dot(n) > size.dot(n) * self.render_split.value
    }

    fn camera(&mut self) -> &mut Camera {
        if self.is_cursor_in_julia() {
            &mut self.cameras.julia
        } else {
            &mut self.cameras.mandelbrot
        }
    }
}

impl ControllerTrait for Controller {
    fn resize(&mut self, size: UVec2) {
        if !self.cameras.has_calibrated {
            if size.x < size.y {
                self.cameras.mandelbrot.translate = vec2(-0.8, 1.0);
                self.cameras.julia.translate = vec2(0.0, -1.0);
            }
            self.cameras.has_calibrated = true;
        }
        self.size = size;
    }

    fn mouse_move(&mut self, position: Vec2) {
        self.prev_cursor = self.cursor;
        self.cursor = position;
        if self.iterations.dragging {
            self.iterations.marker += self.to_uv(self.cursor) - self.to_uv(self.prev_cursor);
            self.iterations.recompute = self.iterations.enabled;
        } else if self.render_split.dragging.is_some() {
            let size = self.size.as_vec2();
            let delta = (self.prev_cursor - self.cursor) / size;
            let value = if size.x > size.y { delta.x } else { delta.y };
            self.render_split.value -= value;
        } else {
            for camera in self.cameras.iter_mut() {
                if camera.grabbing {
                    self.context_menu = None;
                    let delta = (self.prev_cursor - self.cursor) / self.size.y as f32;
                    camera.translate += delta / camera.zoom;
                }
            }
        }
    }

    fn mouse_scroll(&mut self, delta: Vec2) {
        let cursor = self.cursor;
        let size = self.size.as_vec2();
        let camera = self.camera();
        let val = delta.y * 0.1;
        let prev_zoom = camera.zoom;
        let mouse_pos0 = ((cursor - size / 2.0) / size.y) / camera.zoom;
        camera.zoom = (prev_zoom * (1.0 + val)).clamp(0.05, MAX_ZOOM);
        let mouse_pos1 = ((cursor - size / 2.0) / size.y) / camera.zoom;
        camera.translate += mouse_pos0 - mouse_pos1;
        self.iterations.recompute = self.iterations.enabled;
        self.num_iterations = calculate_num_iterations(
            self.cameras.mandelbrot.zoom,
            self.additional_iterations as f32,
        );
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
        let fragment_constants = FragmentConstants {
            size: self.size.into(),
            time: self.start.elapsed().as_secs_f32(),
            cursor: self.cursor,
            prev_cursor: self.prev_cursor,
            mandelbrot_camera_translate: self.cameras.mandelbrot.translate,
            mandelbrot_camera_zoom: self.cameras.mandelbrot.zoom,
            julia_camera_translate: self.cameras.julia.translate,
            julia_camera_zoom: self.cameras.julia.zoom,
            num_iterations: self.num_iterations,
            show_iterations: (self.iterations.enabled && self.iterations.points.len() > 0).into(),
            num_points: self.iterations.points.len() as u32,
            marker: self.iterations.marker,
            render_julia_set: self.render_julia_set.into(),
            render_split: self.render_split.value,
            palette: self.palette,
            smooth_factor: self.smooth.factor(),
            animate_time: self.animate.value,
            palette_period: self.palette_period,
            render_style: self.render_style,
            padding: 0,
        };
        fragment_constants
    }

    fn describe_buffers(&self) -> Vec<easy_shader_runner::BufferDescriptor> {
        use easy_shader_runner::wgpu;
        vec![easy_shader_runner::BufferDescriptor {
            data: &[0; std::mem::size_of::<Vec2>() * MAX_ITER_POINTS as usize],
            read_only: true,
            shader_stages: wgpu::ShaderStages::FRAGMENT,
            cpu_writable: true,
        }]
    }

    fn receive_buffers(&mut self, mut buffers: Vec<wgpu::Buffer>) {
        debug_assert!(buffers.len() == 1);
        self.iterations.points_buffer = Some(buffers.swap_remove(0));
    }

    fn ui(
        &mut self,
        ctx: &egui::Context,
        ui_state: &UiState,
        graphics_context: &easy_shader_runner::GraphicsContext,
    ) {
        self.ui_impl(ctx, ui_state, graphics_context);
    }
}

fn calculate_num_iterations(zoom: f32, c: f32) -> f32 {
    ((zoom + 1.0).log2() * 9.0 + c).max(1.0)
}
