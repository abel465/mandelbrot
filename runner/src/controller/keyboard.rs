use easy_shader_runner::winit::{
    event::KeyEvent,
    keyboard::{Key, NamedKey},
};

impl super::Controller {
    pub fn keyboard_input_impl(&mut self, key: KeyEvent) {
        if !key.state.is_pressed() {
            match key.logical_key {
                Key::Named(NamedKey::ArrowDown) => {
                    self.delta_params.translate.y = self.delta_params.translate.y.min(0.0);
                }
                Key::Named(NamedKey::ArrowUp) => {
                    self.delta_params.translate.y = self.delta_params.translate.y.max(0.0);
                }
                Key::Named(NamedKey::ArrowLeft) => {
                    self.delta_params.translate.x = self.delta_params.translate.x.max(0.0);
                }
                Key::Named(NamedKey::ArrowRight) => {
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
                            self.marker_iterations.recompute = self.marker_iterations.enabled;
                            self.mandelbrot_reference.recompute = true;
                            self.cameras.mandelbrot.needs_reiterate = true;
                            self.cameras.julia.needs_reiterate = true;
                        }
                        'H' => {
                            self.exponent = self.exponent.floor() + 1.0;
                            self.marker_iterations.recompute = self.marker_iterations.enabled;
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
            Key::Named(NamedKey::ArrowDown) => {
                self.delta_params.translate.y = move_speed;
            }
            Key::Named(NamedKey::ArrowUp) => {
                self.delta_params.translate.y = -move_speed;
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.delta_params.translate.x = -move_speed;
            }
            Key::Named(NamedKey::ArrowRight) => {
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
}
