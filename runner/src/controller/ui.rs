use super::Controller;
use easy_shader_runner::{egui, UiState};
use glam::*;
use push_constants::shader::*;
use shared::*;
use web_time::Instant;

impl Controller {
    pub fn ui_impl(
        &mut self,
        ctx: &egui::Context,
        ui_state: &mut UiState,
        graphics_context: &easy_shader_runner::GraphicsContext,
    ) {
        self.handle_param_deltas();
        self.iterations.mode = if self.cameras.mandelbrot.zoom > 1000.0 {
            IterationMode::Perturbation
        } else {
            IterationMode::Regular
        };

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
        if self.cameras.mandelbrot.needs_reiterate
            && matches!(self.iterations.mode, IterationMode::Perturbation)
        {
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
        let mut prev_norm;
        let mut norm = 0.0;
        for i in 0..self.num_iterations as u32 {
            prev_prev_z = prev_z;
            prev_z = z;
            z = z * z + c;
            stats.angle_sum += angle_between_three_points(prev_prev_z.0, prev_z.0, z.0);
            stats.distance_sum += prev_z.distance(z.0);
            prev_norm = norm;
            norm = z.norm();
            if z.0 == prev_z.0 {
                break;
            }
            self.iterations.points.push(z.0);
            if norm >= 2.0 {
                stats.final_distance = prev_z.distance(z.0);
                stats.final_angle = z.arg();
                stats.count = i + 1;
                stats.final_norm = norm;
                stats.proximity = get_proximity(prev_norm, norm);
                while norm < 1e4 {
                    z = z * z + c;
                    norm = z.norm();
                    self.iterations.points.push(z.0);
                }
                break;
            }
            if i + 1 == self.num_iterations as u32 {
                stats.final_distance = prev_z.distance(z.0);
                stats.final_angle = z.arg();
                stats.count = i + 1;
                stats.final_norm = norm;
                stats.proximity = get_proximity(prev_norm, norm);
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

    fn handle_param_deltas(&mut self) {
        let dt = self.last_instant.elapsed().as_secs_f64();
        self.last_instant = Instant::now();
        if self.delta_params.zoom != 0.0 {
            self.cameras.mandelbrot.zoom *= (self.delta_params.zoom - 1.0) * dt + 1.0;
            self.cameras.mandelbrot.needs_reiterate = true;
            self.cameras.julia.needs_reiterate = true;
            self.mandelbrot_reference.recompute = true;
            self.iterations.recompute = self.iterations.enabled;
            self.num_iterations = super::calculate_num_iterations(
                self.cameras.mandelbrot.zoom,
                self.additional_iterations as f64,
            );
        }
        if self.delta_params.translate.x != 0.0 || self.delta_params.translate.y != 0.0 {
            self.cameras.mandelbrot.translate +=
                self.delta_params.translate / self.cameras.mandelbrot.zoom * dt;
            self.cameras.mandelbrot.needs_reiterate = true;
            self.mandelbrot_reference.recompute = true;
        }
        if self.delta_params.period != 0.0 {
            self.palette_period *= (self.delta_params.period as f32 - 1.0) * dt as f32 + 1.0;
        }
        if self.delta_params.animation_speed != 0.0 {
            self.animate.speed *=
                (self.delta_params.animation_speed as f32 - 1.0) * dt as f32 + 1.0;
        }
        if self.delta_params.iterations != 0.0 {
            self.additional_iterations += self.delta_params.iterations * dt;
            self.cameras.mandelbrot.needs_reiterate = true;
            self.cameras.julia.needs_reiterate = true;
            self.mandelbrot_reference.recompute = true;
            self.iterations.recompute = self.iterations.enabled;
            self.num_iterations = super::calculate_num_iterations(
                self.cameras.mandelbrot.zoom,
                self.additional_iterations as f64,
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
                    self.cameras.julia.needs_reiterate = true;
                }
            });
        if let Some(r) = r {
            if r.response.clicked_elsewhere() {
                self.context_menu = None;
            }
        }
    }

    fn render_partition_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("Render Partitioning").size(15.0));
        });
        egui::Grid::new("render_partitioning_grid").show(ui, |ui| {
            let render_partioning = self.render_partitioning;
            ui.radio_value(
                &mut self.render_partitioning,
                RenderPartitioning::Outside,
                "Outside",
            );
            ui.radio_value(
                &mut self.render_partitioning,
                RenderPartitioning::Inside,
                "Inside",
            );
            ui.end_row();
            ui.radio_value(
                &mut self.render_partitioning,
                RenderPartitioning::Both,
                "Both",
            );
            ui.end_row();
            if self.render_partitioning != render_partioning {
                self.cameras.mandelbrot.needs_reiterate = true;
                self.cameras.julia.needs_reiterate = true;
            }
        });
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
                    ui.radio_value(&mut self.palette, Palette::Pastel, "Pastel");
                    ui.radio_value(&mut self.palette, Palette::SolarizedDark, "SolarizedDark");
                    ui.end_row();
                    ui.radio_value(&mut self.palette, Palette::Copper, "Copper");
                    ui.radio_value(&mut self.palette, Palette::RedAndBlack, "RedAndBlack");
                    ui.end_row();
                    ui.radio_value(&mut self.palette, Palette::NeonA, "NeonA");
                    ui.radio_value(&mut self.palette, Palette::Highlighter, "Highlighter");
                    ui.end_row();
                    ui.radio_value(&mut self.palette, Palette::NeonB, "NeonB");
                    ui.radio_value(&mut self.palette, Palette::RGB, "RGB");
                    ui.end_row();
                    ui.radio_value(&mut self.palette, Palette::NeonC, "NeonC");
                    ui.radio_value(&mut self.palette, Palette::Zebra, "Zebra");
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
                        RenderStyle::FinalDistance,
                        "Final Distance",
                    );
                    ui.end_row();
                    ui.radio_value(
                        &mut self.render_style,
                        RenderStyle::FinalAngle,
                        "Final Angle",
                    );
                    ui.radio_value(
                        &mut self.render_style,
                        RenderStyle::DistanceSum,
                        "Distance Sum",
                    );
                    ui.end_row();
                    ui.radio_value(&mut self.render_style, RenderStyle::NormSum, "Norm Sum");
                    ui.end_row();
                    if self.render_style != render_style {
                        self.cameras.mandelbrot.needs_reiterate = true;
                        self.cameras.julia.needs_reiterate = true;
                    }
                });
                ui.separator();
                self.render_partition_ui(ui);
                ui.separator();
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Additional Iterations").size(14.0));
                });
                if ui
                    .add(egui::Slider::new(
                        &mut self.additional_iterations,
                        0.0..=super::MAX_ADDITIONAL_ITERS as f64,
                    ))
                    .changed()
                {
                    self.num_iterations = super::calculate_num_iterations(
                        self.cameras.mandelbrot.zoom,
                        self.additional_iterations as f64,
                    );
                    self.iterations.recompute = self.iterations.enabled;
                    self.mandelbrot_reference.recompute = true;
                    self.cameras.mandelbrot.needs_reiterate = true;
                    self.cameras.julia.needs_reiterate = true;
                };
                ui.separator();
                ui.toggle_value(&mut self.smooth.enable, "Smooth");
                if ui
                    .add_enabled(
                        self.smooth.enable,
                        egui::Slider::new(&mut self.smooth.value, 0.0..=1.0),
                    )
                    .changed()
                {
                    self.cameras.mandelbrot.needs_reiterate = true;
                    self.cameras.julia.needs_reiterate = true;
                }
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
                    if self.render_julia_set {
                        self.cameras.julia.needs_reiterate = true;
                    } else {
                        self.cameras.mandelbrot.needs_reiterate = true;
                    }
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

                    ui.label("iteration mode");
                    ui.monospace(format!("{:?}", self.iterations.mode));
                    ui.end_row();

                    if self.iterations.enabled {
                        ui.label("num iterations");
                        ui.monospace(format!("{:.2}", self.iterations.stats.count));
                        ui.end_row();

                        ui.label("proximity");
                        ui.monospace(format!("{:.4}", self.iterations.stats.proximity));
                        ui.end_row();
                    }
                });
                if self.iterations.enabled {
                    egui::Grid::new("iterations_debug_grid").show(ui, |ui| {
                        ui.label("");
                        ui.label("final");
                        ui.label("sum");
                        ui.label("average");
                        ui.end_row();
                        ui.label("|z|");
                        ui.monospace(format!("{:.4}", self.iterations.stats.final_norm));
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                        ui.label("angle");
                        ui.monospace(format!("{:.4}", self.iterations.stats.final_angle));
                        ui.monospace(format!("{:.4}", self.iterations.stats.angle_sum));
                        ui.label("");
                        ui.end_row();
                        ui.label("distance");
                        ui.monospace(format!("{:.4}", self.iterations.stats.final_distance));
                        ui.monospace(format!("{:.4}", self.iterations.stats.distance_sum));
                        ui.end_row();
                    });
                }
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
