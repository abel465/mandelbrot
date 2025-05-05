use crate::Options;
use easy_shader_runner::{egui, wgpu, winit, ControllerTrait, UiState};
use glam::*;
use shared::push_constants::shader::*;
use shared::MARKER_RADIUS;
use web_time::Instant;
use winit::event::{ElementState, MouseButton};

const MAX_ITER_POINTS: u32 = 100;

struct Camera {
    zoom: f32,
    translate: Vec2,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            translate: vec2(-1.0, 1.0 / 6.0),
        }
    }
}

#[derive(Default)]
struct Iterations {
    enabled: bool,
    dragging: bool,
    marker: Vec2,
    points: Vec<Vec2>,
    points_buffer: Option<wgpu::Buffer>,
    recompute: bool,
}

pub struct Controller {
    size: UVec2,
    start: Instant,
    cursor: Vec2,
    prev_cursor: Vec2,
    mouse_button_pressed: u32,
    camera: Camera,
    debug: bool,
    num_iterations: f32,
    style: RenderStyle,
    iterations: Iterations,
    context_menu: Option<Vec2>,
}

impl Controller {
    pub fn new(options: &Options) -> Self {
        let now = Instant::now();

        Self {
            size: UVec2::ZERO,
            start: now,
            cursor: Vec2::ZERO,
            prev_cursor: Vec2::ZERO,
            mouse_button_pressed: 0,
            camera: Default::default(),
            debug: options.debug,
            num_iterations: 50.0,
            style: RenderStyle::default(),
            iterations: Default::default(),
            context_menu: None,
        }
    }

    fn to_uv(&self, p: Vec2) -> Vec2 {
        let size = self.size.as_vec2();
        self.camera.translate
            + (p - 0.5 * size) * vec2(size.x / size.y, 1.0) / self.camera.zoom / size
    }
}

impl ControllerTrait for Controller {
    fn resize(&mut self, size: UVec2) {
        self.size = size;
    }

    fn mouse_move(&mut self, position: Vec2) {
        self.prev_cursor = self.cursor;
        self.cursor = position;
        if self.iterations.dragging {
            self.iterations.marker += self.to_uv(self.cursor) - self.to_uv(self.prev_cursor);
            self.iterations.recompute = true;
        } else if self.mouse_button_pressed & 1 == 1 {
            // is dragging
            self.context_menu = None;
            self.camera.translate +=
                (self.prev_cursor - self.cursor) / self.size.y as f32 / self.camera.zoom;
            self.camera.translate = self
                .camera
                .translate
                .clamp(vec2(-2.0, -2.0), vec2(1.0, 2.0));
        }
    }

    fn mouse_scroll(&mut self, delta: Vec2) {
        let val = delta.y * 0.1;
        let prev_zoom = self.camera.zoom;
        let mouse_pos0 =
            ((self.cursor - self.size.as_vec2() / 2.0) / self.size.y as f32) / self.camera.zoom;
        self.camera.zoom = (prev_zoom * (1.0 + val)).clamp(0.1, 10000.0);
        let mouse_pos1 =
            ((self.cursor - self.size.as_vec2() / 2.0) / self.size.y as f32) / self.camera.zoom;
        self.camera.translate += mouse_pos0 - mouse_pos1;
        self.camera.translate = self
            .camera
            .translate
            .clamp(vec2(-2.0, -2.0), vec2(1.0, 2.0));
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        let mask = 1
            << match button {
                MouseButton::Left => {
                    if matches!(state, ElementState::Pressed) {
                        self.iterations.dragging =
                            self.to_uv(self.cursor).distance(self.iterations.marker)
                                < MARKER_RADIUS / self.camera.zoom;
                    } else {
                        self.iterations.dragging = false;
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
        self.num_iterations = (self.camera.zoom * 10000000000000.0).log2() * 5.0 - 190.0;
        let fragment_constants = FragmentConstants {
            size: self.size.into(),
            time: self.start.elapsed().as_secs_f32(),
            cursor: self.cursor,
            prev_cursor: self.prev_cursor,
            camera_translate: self.camera.translate,
            camera_zoom: self.camera.zoom,
            num_iterations: self.num_iterations,
            style: self.style,
            show_iterations: (self.iterations.enabled && self.iterations.points.len() > 0).into(),
            num_points: self.iterations.points.len() as u32,
            iterations_marker: self.iterations.marker,
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
        _ui_state: &UiState,
        graphics_context: &easy_shader_runner::GraphicsContext,
    ) {
        let width = if self.debug { 120.0 } else { 80.0 };
        if let Some(pos) = self.context_menu {
            let r = egui::Window::new("right_click_menu")
                .frame(egui::Frame::none())
                .title_bar(false)
                .resizable(false)
                .fixed_pos(egui::pos2(pos.x, pos.y))
                .show(ctx, |ui| {
                    if ui.button("Show iterations here").clicked() {
                        self.iterations.marker = self.to_uv(pos);
                        self.iterations.enabled = true;
                        self.iterations.recompute = true;
                        self.context_menu = None;
                    }
                });
            if let Some(r) = r {
                if r.response.clicked_elsewhere() {
                    self.context_menu = None;
                }
            }
        }
        egui::Window::new("ui")
            .min_width(width)
            .max_width(width)
            .resizable(false)
            .show(ctx, |ui| {
                let uv_cursor = self.to_uv(self.cursor);
                if self.iterations.dragging {
                    ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
                } else if self.iterations.enabled
                    && uv_cursor.distance(self.iterations.marker) < MARKER_RADIUS / self.camera.zoom
                {
                    ctx.set_cursor_icon(egui::CursorIcon::Grab);
                } else {
                    ctx.set_cursor_icon(egui::CursorIcon::Default);
                }

                ui.radio_value(&mut self.style, RenderStyle::RedGlow, "Red Glow");
                ui.radio_value(&mut self.style, RenderStyle::Circus, "Circus");
                ui.separator();
                if ui
                    .checkbox(&mut self.iterations.enabled, "Show Iterations")
                    .clicked()
                    && self.iterations.enabled
                {
                    self.iterations.marker = self.camera.translate;
                    self.iterations.recompute = true;
                };
                if self.iterations.recompute {
                    use shared::complex::Complex;
                    self.iterations.points.clear();
                    let c = Complex::from(self.iterations.marker);
                    let mut z = Complex::ZERO;
                    for _ in 0..(self.num_iterations as u32).min(MAX_ITER_POINTS) {
                        z = z * z + c;
                        if self.iterations.points.last().is_some_and(|p| p == &z.0)
                            || z.x.abs() > 10.0
                            || z.y.abs() > 10.0
                        {
                            break;
                        }
                        self.iterations.points.push(z.0);
                    }
                    if self.iterations.enabled && self.iterations.points.len() > 0 {
                        graphics_context.queue.write_buffer(
                            self.iterations.points_buffer.as_ref().unwrap(),
                            0,
                            bytemuck::cast_slice(&self.iterations.points),
                        );
                    }
                    self.iterations.recompute = false;
                }
                ui.separator();
                ui.checkbox(&mut self.debug, "Debug");
                if self.debug {
                    egui::Grid::new("debug_grid").show(ui, |ui| {
                        ui.label("Zoom");
                        ui.monospace(format!("{:.1}x", self.camera.zoom));
                        ui.end_row();

                        ui.label("X");
                        ui.monospace(format!("{:+.6}", self.camera.translate.x));
                        ui.end_row();

                        ui.label("Y");
                        ui.monospace(format!("{:+.6}", self.camera.translate.y));
                        ui.end_row();

                        ui.label("Iterations");
                        ui.monospace(format!("{:.2}", self.num_iterations));
                        ui.end_row();

                        ui.label("cursor X");
                        ui.monospace(format!("{:+.6}", uv_cursor.x));
                        ui.end_row();

                        ui.label("cursor Y");
                        ui.monospace(format!("{:+.6}", uv_cursor.y));
                        ui.end_row();

                        if self.iterations.enabled {
                            ui.label("marker X");
                            ui.monospace(format!("{:+.6}", self.iterations.marker.x));
                            ui.end_row();

                            ui.label("marker Y");
                            ui.monospace(format!("{:+.6}", self.iterations.marker.y));
                            ui.end_row();
                        }
                    });
                }
            });
    }
}
