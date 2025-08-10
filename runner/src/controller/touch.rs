use std::time::Duration;

use super::*;

#[derive(Clone, Copy, Debug)]
pub enum TouchType {
    Mandelbrot,
    Julia,
    RenderSplit,
    Marker,
}

#[derive(Clone, Copy)]
pub struct Touch {
    pos: DVec2,
    touch_type: TouchType,
    has_moved: bool,
    instant: Instant,
}

impl Touch {
    fn new(pos: DVec2, touch_type: TouchType) -> Self {
        Self {
            pos,
            touch_type,
            instant: Instant::now(),
            has_moved: false,
        }
    }
}

impl Controller {
    fn handle_move(&mut self, id: u64, touch: Touch, last_position: DVec2, position: DVec2) {
        let size = self.size.as_dvec2();
        let delta = (last_position - position) / self.size.y as f64;
        if delta.x != 0.0 || delta.y != 0.0 {
            match touch.touch_type {
                TouchType::Mandelbrot => {
                    let delta = BigVec2::from_dvec2(delta).with_precision(PRECISION);
                    self.cameras.mandelbrot.translate += delta / self.cameras.mandelbrot.zoom;
                    self.cameras.mandelbrot.needs_reiterate = true;
                }
                TouchType::Julia => {
                    let delta = BigVec2::from_dvec2(delta).with_precision(PRECISION);
                    self.cameras.julia.translate += delta / self.cameras.julia.zoom;
                    self.cameras.julia.needs_reiterate = true;
                }
                TouchType::RenderSplit => {
                    let delta = (last_position - position) / size;
                    let value = if size.x > size.y { delta.x } else { delta.y };
                    self.render_split.value -= value;
                    if value > 0.0 {
                        self.cameras.julia.needs_reiterate = true;
                    } else if value < 0.0 {
                        self.cameras.mandelbrot.needs_reiterate = true;
                    }
                }
                TouchType::Marker => {
                    self.marker_iterations.position +=
                        self.to_uv_space_big(position) - self.to_uv_space_big(last_position);
                    self.marker_iterations.recompute = self.marker_iterations.enabled;
                    self.cameras.julia.needs_reiterate = true;
                }
            }
            self.touches.get_mut(&id).unwrap().pos = position;
        }
    }

    fn handle_pinch(&mut self, id: u64, touch: Touch, last_position: DVec2, position: DVec2) {
        let size = self.size.as_dvec2();
        let other_touch = self.touches.iter().find(|(i, _)| **i != id).unwrap().1;
        let last_distance = last_position.distance(other_touch.pos);
        let this_distance = position.distance(other_touch.pos);

        let val = (this_distance - last_distance) / self.size.y as f64 * 3.0;
        let pinch_to_zoom = |camera: &mut Camera, max_zoom: f64| {
            let avg_pos = (last_position + position) / 2.0;
            let avg_pos0 = BigVec2::from_dvec2(avg_pos - size / 2.0) / camera.zoom / size.y;
            camera.zoom = (camera.zoom * (1.0 + val)).clamp(0.05, max_zoom);
            let avg_pos1 = BigVec2::from_dvec2(avg_pos - size / 2.0) / camera.zoom / size.y;
            camera.translate += avg_pos0 - avg_pos1;
            camera.needs_reiterate = true;
        };
        match touch.touch_type {
            TouchType::Mandelbrot => {
                pinch_to_zoom(&mut self.cameras.mandelbrot, MAX_ZOOM_MANDELBROT);
                self.cameras.julia.needs_reiterate = true;
                self.touches.get_mut(&id).unwrap().pos = position;
            }
            TouchType::Julia => {
                pinch_to_zoom(&mut self.cameras.julia, MAX_ZOOM_JULIA);
                self.touches.get_mut(&id).unwrap().pos = position;
            }
            _ => {}
        }
    }

    pub fn touch_impl(&mut self, id: u64, phase: TouchPhase, position: DVec2) {
        match phase {
            TouchPhase::Started => {
                let touch_type = if self.pos_on_render_split(position).is_some() {
                    TouchType::RenderSplit
                } else if self.pos_on_marker(position) {
                    TouchType::Marker
                } else if self.is_pos_in_julia(position) {
                    TouchType::Julia
                } else {
                    TouchType::Mandelbrot
                };
                self.touches.insert(id, Touch::new(position, touch_type));
            }
            TouchPhase::Moved => {
                self.context_menu = None;
                let Some(touch) = self.touches.get_mut(&id).map(|touch| {
                    touch.has_moved = true;
                    *touch
                }) else {
                    return;
                };
                let is_pinch = self.touches.len() > 1;
                let last_position = touch.pos;
                if is_pinch {
                    self.handle_pinch(id, touch, last_position, position);
                } else {
                    self.handle_move(id, touch, last_position, position);
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                let Some(touch) = self.touches.remove(&id) else {
                    return;
                };
                if touch.instant.elapsed() > Duration::from_millis(700) && !touch.has_moved {
                    self.context_menu = Some(position);
                }
            }
        };
    }
}
