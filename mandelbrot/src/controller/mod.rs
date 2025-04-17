use crate::Options;
use easy_shader_runner::{egui, winit, ControllerTrait, UiState, UserEvent};
use glam::*;
use shared::push_constants::shader::*;
use web_time::Instant;
use winit::event::{ElementState, MouseButton};

struct Camera {
    zoom: f32,
    translate: Vec2,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            translate: Default::default(),
        }
    }
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
        }
    }
}

impl ControllerTrait for Controller {
    fn resize(&mut self, size: UVec2) {
        self.size = size;
    }

    fn mouse_move(&mut self, position: Vec2) {
        self.prev_cursor = self.cursor;
        self.cursor = position;
        if self.mouse_button_pressed & 1 == 1 {
            // is dragging
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
                MouseButton::Left => 0,
                MouseButton::Middle => 1,
                MouseButton::Right => 2,
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
        };
        fragment_constants
    }

    fn ui<F: Fn(UserEvent)>(&mut self, ctx: &egui::Context, _ui_state: &UiState, _send_event: F) {
        let width = if self.debug { 120.0 } else { 80.0 };
        egui::Window::new("ui")
            .min_width(width)
            .max_width(width)
            .resizable(false)
            .show(ctx, |ui| {
                ui.radio_value(&mut self.style, RenderStyle::RedGlow, "Red Glow");
                ui.radio_value(&mut self.style, RenderStyle::Circus, "Circus");
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
                    });
                }
            });
    }
}
