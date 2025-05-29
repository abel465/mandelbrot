use super::Controller;
use easy_shader_runner::{egui, UiState};
use glam::*;
use push_constants::shader::*;
use shared::*;

impl Controller {
    pub fn ui_impl(
        &mut self,
        ctx: &egui::Context,
        ui_state: &UiState,
        graphics_context: &easy_shader_runner::GraphicsContext,
    ) {
        if let Some(pos) = self.context_menu {
            self.context_menu_window(ctx, pos);
        }
        self.main_window(ctx);
        self.handle_cursor_icon(ctx);

        if self.iterations.recompute {
            self.recompute_iterations(graphics_context);
        }
        if self.show_fps {
            self.fps_window(ctx, ui_state);
        }
        if self.debug {
            self.debug_window(ctx);
        }
    }

    fn recompute_iterations(&mut self, graphics_context: &easy_shader_runner::GraphicsContext) {
        use shared::complex::Complex;
        debug_assert!(self.iterations.enabled);
        self.iterations.points.clear();
        let c = Complex::from(self.iterations.marker);
        let mut z = Complex::ZERO;
        let mut stats = super::IterationStats::default();
        let mut prev_z = Complex::new(-1.0, 0.0);
        let mut prev_prev_z;
        let mut norm_sq;
        for i in 0..self.num_iterations as u32 {
            prev_prev_z = prev_z;
            prev_z = z;
            z = z * z + c;
            stats.total_angle += angle_between_three_points(prev_prev_z.0, prev_z.0, z.0);
            stats.total_distance += prev_z.distance(z.0);
            norm_sq = z.norm_squared();
            stats.last_norm_sq = norm_sq;
            if norm_sq >= 4.0 {
                stats.count = i;
                stats.last_distance = prev_z.distance(z.0);
                stats.last_angle = z.arg();
                stats.proximity = get_lerp_factor(prev_z.norm_squared(), norm_sq);
                while norm_sq < 1e9 {
                    z = z * z + c;
                    norm_sq = z.norm_squared();
                    self.iterations.points.push(z.0);
                }
                break;
            }
            if i + 1 == self.num_iterations as u32 {
                stats.last_distance = prev_z.distance(z.0);
                stats.last_angle = z.arg();
            }
            if z.0 == prev_z.0 {
                break;
            }
            self.iterations.points.push(z.0);
        }
        self.iterations.stats = stats;

        if self.iterations.points.len() > 0 {
            graphics_context.queue.write_buffer(
                self.iterations.points_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&self.iterations.points),
            );
        }
        self.iterations.recompute = false;
    }

    fn handle_cursor_icon(&mut self, ctx: &egui::Context) {
        if self.iterations.dragging {
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        } else if let Some(icon) = self.render_split.dragging {
            ctx.set_cursor_icon(icon);
        } else if self.can_grab_marker() {
            ctx.set_cursor_icon(egui::CursorIcon::Grab);
        } else if let Some(icon) = self.can_grab_render_split() {
            ctx.set_cursor_icon(icon);
        } else {
            ctx.set_cursor_icon(egui::CursorIcon::Default);
        }
    }

    fn context_menu_window(&mut self, ctx: &egui::Context, pos: Vec2) {
        let r = egui::Window::new("right_click_menu")
            .frame(egui::Frame::NONE)
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

    fn main_window(&mut self, ctx: &egui::Context) {
        let width = 120.0;
        egui::Window::new("ui")
            .min_width(width)
            .max_width(width)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Palette").size(15.0));
                });
                egui::Grid::new("col_grid").show(ui, |ui| {
                    ui.radio_value(&mut self.palette, Palette::A, "A");
                    ui.radio_value(&mut self.palette, Palette::B, "B");
                    ui.radio_value(&mut self.palette, Palette::C, "C");
                    ui.end_row();
                    ui.radio_value(&mut self.palette, Palette::D, "D");
                    ui.radio_value(&mut self.palette, Palette::E, "E");
                    ui.radio_value(&mut self.palette, Palette::F, "F");
                    ui.end_row();
                });
                ui.label("Period");
                ui.add(egui::Slider::new(&mut self.palette_period, 0.01..=1.0));
                ui.separator();
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Render Style").size(15.0));
                });
                egui::Grid::new("render_style_grid").show(ui, |ui| {
                    ui.radio_value(
                        &mut self.render_style,
                        RenderStyle::Iterations,
                        "Iterations",
                    );
                    ui.radio_value(&mut self.render_style, RenderStyle::Arg, "Arg");
                    ui.end_row();
                    ui.radio_value(&mut self.render_style, RenderStyle::Distance, "Distance");
                    ui.end_row();
                });
                ui.separator();
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Additional Iterations").size(14.0));
                });
                if ui
                    .add(egui::Slider::new(
                        &mut self.additional_iterations,
                        0..=super::MAX_ADDITIONAL_ITERS,
                    ))
                    .changed()
                {
                    self.num_iterations = super::calculate_num_iterations(
                        self.cameras.mandelbrot.zoom,
                        self.additional_iterations as f32,
                    );
                    self.iterations.recompute = self.iterations.enabled;
                };
                ui.separator();
                ui.toggle_value(&mut self.smooth.enable, "Smooth");
                ui.add_enabled(
                    self.smooth.enable,
                    egui::Slider::new(&mut self.smooth.value, 0.0..=1.0),
                );
                ui.horizontal(|ui| {
                    ui.toggle_value(&mut self.animate.enable, "Animate");
                    if ui
                        .add_enabled(
                            self.animate.enable,
                            egui::SelectableLabel::new(self.animate.reverse, "Reverse"),
                        )
                        .clicked()
                    {
                        self.animate.reverse = !self.animate.reverse;
                    }
                });
                ui.add_enabled(
                    self.animate.enable,
                    egui::Slider::new(&mut self.animate.speed, 0.0..=1.0),
                );
                ui.separator();
                if ui
                    .checkbox(&mut self.iterations.enabled, "Marker Iterations")
                    .clicked()
                    && self.iterations.enabled
                {
                    self.iterations.recompute = true;
                };
                ui.checkbox(&mut self.render_julia_set, "Render Julia Set");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.debug, "Debug");
                    ui.add_space(12.0);
                    ui.checkbox(&mut self.show_fps, "Show FPS");
                });
            });
    }

    fn fps_window(&mut self, ctx: &egui::Context, ui_state: &UiState) {
        egui::Window::new("fps")
            .title_bar(false)
            .resizable(false)
            .interactable(false)
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::splat(-10.0))
            .show(ctx, |ui| {
                ui.label(format!("FPS: {}", ui_state.fps));
            });
    }

    fn debug_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("debug_window")
            .title_bar(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.collapsing("Camera", |ui| {
                    egui::Grid::new("debug_camera_grid").show(ui, |ui| {
                        {
                            let camera = &self.cameras.mandelbrot;
                            ui.label("Mandelbrot Zoom");
                            let zoom = camera.zoom;
                            if zoom < 1000.0 {
                                ui.monospace(format!("{:.2}x", zoom));
                            } else {
                                ui.monospace(format!("{:.1}x", zoom));
                            }
                            ui.end_row();

                            ui.label("Mandelbrot X");
                            ui.monospace(format!("{:+.6}", camera.translate.x));
                            ui.end_row();

                            ui.label("Mandelbrot Y");
                            ui.monospace(format!("{:+.6}", camera.translate.y));
                            ui.end_row();
                        }

                        {
                            let camera = &self.cameras.julia;
                            ui.label("Julia Zoom");
                            let zoom = self.cameras.julia.zoom;
                            if zoom < 1000.0 {
                                ui.monospace(format!("{:.2}x", zoom));
                            } else {
                                ui.monospace(format!("{:.1}x", zoom));
                            }
                            ui.end_row();

                            ui.label("Julia X");
                            ui.monospace(format!("{:+.6}", camera.translate.x));
                            ui.end_row();

                            ui.label("Julia Y");
                            ui.monospace(format!("{:+.6}", camera.translate.y));
                            ui.end_row();
                        }

                        {
                            let cursor_uv = self.to_uv(self.cursor);
                            ui.label("cursor X");
                            ui.monospace(format!("{:+.6}", cursor_uv.x));
                            ui.end_row();

                            ui.label("cursor Y");
                            ui.monospace(format!("{:+.6}", cursor_uv.y));
                            ui.end_row();
                        }

                        if self.iterations.enabled || self.render_julia_set {
                            ui.label("marker X");
                            ui.monospace(format!("{:+.6}", self.iterations.marker.x));
                            ui.end_row();

                            ui.label("marker Y");
                            ui.monospace(format!("{:+.6}", self.iterations.marker.y));
                            ui.end_row();
                        }
                    });
                });

                egui::Grid::new("debug_grid").show(ui, |ui| {
                    ui.label("max iterations");
                    ui.monospace(format!("{:.2}", self.num_iterations));
                    ui.end_row();

                    if self.iterations.enabled {
                        ui.label("num iterations");
                        ui.monospace(format!("{:.2}", self.iterations.stats.count));
                        ui.end_row();

                        ui.label("last |z|²");
                        ui.monospace(format!("{:.4}", self.iterations.stats.last_norm_sq));
                        ui.end_row();

                        ui.label("last angle");
                        ui.monospace(format!("{:.4}", self.iterations.stats.last_angle));
                        ui.end_row();

                        ui.label("total angle");
                        ui.monospace(format!("{:.4}", self.iterations.stats.total_angle));
                        ui.end_row();

                        ui.label("last distance");
                        ui.monospace(format!("{:.4}", self.iterations.stats.last_distance));
                        ui.end_row();

                        ui.label("total distance");
                        ui.monospace(format!("{:.4}", self.iterations.stats.total_distance));
                        ui.end_row();

                        ui.label("proximity");
                        ui.monospace(format!("{:.4}", self.iterations.stats.proximity));
                        ui.end_row();
                    }
                });
            });
    }
}

fn angle_between_three_points(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    fn _cross(a: Vec2, b: Vec2) -> f32 {
        a.x * b.y - a.y * b.x
    }
    let ab = b - a;
    let bc = c - b;
    (_cross(ab, bc)).atan2(ab.dot(bc))
}
