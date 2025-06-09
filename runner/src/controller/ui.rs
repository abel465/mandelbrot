use super::Controller;
use easy_shader_runner::{egui, UiState};
use glam::*;
use push_constants::shader::*;
use shared::*;

impl Controller {
    pub fn ui_impl(
        &mut self,
        ctx: &egui::Context,
        ui_state: &mut UiState,
        graphics_context: &easy_shader_runner::GraphicsContext,
    ) {
        if let Some(pos) = self.context_menu {
            self.context_menu_window(ctx, pos);
        }
        self.main_window(
            ctx,
            #[cfg(not(target_arch = "wasm32"))]
            ui_state,
        );
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
        if self.mandelbrot_reference.recompute {
            self.recompute_reference_iterations(graphics_context);
        }
    }

    fn recompute_reference_iterations(
        &mut self,
        graphics_context: &easy_shader_runner::GraphicsContext,
    ) {
        use crate::big_complex::Complex;
        const FOUR: dashu::float::FBig = dashu::fbig!(100);

        self.mandelbrot_reference.points.clear();
        let c: Complex = self.cameras.mandelbrot.translate.clone().into();
        let mut z = Complex::ZERO.with_precision(128);
        let mut i = 0;
        while i < self.num_iterations as u32 {
            self.mandelbrot_reference.points.push(z.as_vec2());
            i += 1;
            z = z.square() + c.clone();
            let norm_sq = z.norm_squared();
            if norm_sq >= FOUR {
                break;
            }
        }
        self.mandelbrot_reference.points.push(z.as_vec2());
        self.mandelbrot_reference.num_ref_iterations = i;
        graphics_context.queue.write_buffer(
            self.mandelbrot_reference.buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&self.mandelbrot_reference.points),
        );
        self.mandelbrot_reference.recompute = false;
    }

    fn recompute_iterations(&mut self, graphics_context: &easy_shader_runner::GraphicsContext) {
        self.iterations.points.clear();
        self.iterations.recompute = false;
        if self.cameras.mandelbrot.zoom > 1e5 {
            return;
        }
        use shared::complex::Complex;
        debug_assert!(self.iterations.enabled);
        let c = Complex::from(self.iterations.marker.as_vec2());
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
            if z.0 == prev_z.0 {
                break;
            }
            self.iterations.points.push(z.0);
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
        }
        self.iterations.stats = stats;

        if !self.iterations.points.is_empty() {
            graphics_context.queue.write_buffer(
                self.iterations.points_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&self.iterations.points),
            );
        }
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

    fn context_menu_window(&mut self, ctx: &egui::Context, pos: DVec2) {
        let r = egui::Window::new("right_click_menu")
            .frame(egui::Frame::none())
            .title_bar(false)
            .resizable(false)
            .fixed_pos([pos.x as f32, pos.y as f32])
            .show(ctx, |ui| {
                if ui.button("Show iterations here").clicked() {
                    self.iterations.marker = self.to_uv_space_big(pos);
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

    fn main_window(
        &mut self,
        ctx: &egui::Context,
        #[cfg(not(target_arch = "wasm32"))] ui_state: &mut UiState,
    ) {
        let width = 120.0;
        egui::Window::new("Mandelbrot")
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
                    let render_style = self.render_style;
                    ui.radio_value(
                        &mut self.render_style,
                        RenderStyle::Iterations,
                        "Iterations",
                    );
                    ui.radio_value(
                        &mut self.render_style,
                        RenderStyle::LastDistance,
                        "Last Distance",
                    );
                    ui.end_row();
                    ui.radio_value(&mut self.render_style, RenderStyle::Arg, "Arg");
                    ui.end_row();
                    if self.render_style != render_style {
                        self.needs_reiterate = true;
                    }
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
                        self.additional_iterations as f64,
                    );
                    self.iterations.recompute = self.iterations.enabled;
                    self.mandelbrot_reference.recompute = true;
                    self.needs_reiterate = true;
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
                if ui
                    .checkbox(&mut self.render_julia_set, "Render Julia Set")
                    .changed()
                {
                    self.needs_reiterate = true;
                }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.debug, "Debug");
                    ui.add_space(12.0);
                    ui.checkbox(&mut self.show_fps, "FPS");
                    #[cfg(not(target_arch = "wasm32"))]
                    ui.checkbox(&mut ui_state.vsync, "VSync");
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
                ui.label(format!("FPS: {}", ui_state.fps()));
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
                                ui.monospace(format!("{:.2}", zoom));
                            } else if zoom < 10000000.0 {
                                ui.monospace(format!("{:.1}", zoom));
                            } else {
                                ui.monospace(format!("{:+.2e}", zoom));
                            }
                            ui.end_row();

                            ui.label("Mandelbrot X");
                            ui.monospace(format!("{:+.6e}", camera.translate.x.to_f32().value()));
                            ui.end_row();

                            ui.label("Mandelbrot Y");
                            ui.monospace(format!("{:+.6e}", camera.translate.y.to_f32().value()));
                            ui.end_row();
                        }

                        {
                            let camera = &self.cameras.julia;
                            ui.label("Julia Zoom");
                            let zoom = self.cameras.julia.zoom;
                            if zoom < 1000.0 {
                                ui.monospace(format!("{:.2}", zoom));
                            } else if zoom < 10000000.0 {
                                ui.monospace(format!("{:.1}", zoom));
                            } else {
                                ui.monospace(format!("{:+.2e}", zoom));
                            }
                            ui.end_row();

                            ui.label("Julia X");
                            ui.monospace(format!("{:+.6e}", camera.translate.x.to_f32().value()));
                            ui.end_row();

                            ui.label("Julia Y");
                            ui.monospace(format!("{:+.6e}", camera.translate.y.to_f32().value()));
                            ui.end_row();
                        }

                        if self.iterations.enabled || self.render_julia_set {
                            ui.label("marker X");
                            ui.monospace(format!(
                                "{:+.6e}",
                                self.iterations.marker.x.to_f32().value()
                            ));
                            ui.end_row();

                            ui.label("marker Y");
                            ui.monospace(format!(
                                "{:+.6e}",
                                self.iterations.marker.y.to_f32().value()
                            ));
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
