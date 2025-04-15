use crate::{controller::ControllerTrait, fps_counter::FpsCounter, user_event::CustomEvent};
use egui::{
    epaint::{textures::TexturesDelta, ClippedPrimitive},
    Context,
};
use egui_winit::{
    winit::{event::WindowEvent, event_loop::EventLoopProxy, window::Window},
    State,
};
use std::sync::Arc;

pub struct UiState {
    pub fps: usize,
    #[cfg(not(target_arch = "wasm32"))]
    pub vsync: bool,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            fps: 0,
            #[cfg(not(target_arch = "wasm32"))]
            vsync: true,
        }
    }
}

pub struct Ui {
    egui_winit_state: State,
    fps_counter: FpsCounter,
}

impl Ui {
    pub fn new(window: Arc<Window>) -> Self {
        let context = Context::default();
        context.options_mut(|w| w.zoom_with_keyboard = false);
        let viewport_id = context.viewport_id();
        let egui_winit_state = State::new(
            context,
            viewport_id,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        Self {
            egui_winit_state,
            fps_counter: FpsCounter::new(),
        }
    }

    pub fn consumes_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.egui_winit_state
            .on_window_event(window, event)
            .consumed
    }

    pub fn prepare<C: ControllerTrait>(
        &mut self,
        window: &Window,
        ui_state: &mut UiState,
        controller: &mut C,
        event_proxy: &EventLoopProxy<CustomEvent<C>>,
    ) -> (Vec<ClippedPrimitive>, TexturesDelta, egui::Rect, f32) {
        ui_state.fps = self.fps_counter.tick();
        let raw_input = self.egui_winit_state.take_egui_input(window);
        let mut available_rect = egui::Rect::NAN;
        let full_output = self.egui_winit_state.egui_ctx().run(raw_input, |ctx| {
            self.ui(ctx, ui_state, controller, event_proxy);
            available_rect = ctx.available_rect();
        });
        self.egui_winit_state
            .handle_platform_output(window, full_output.platform_output);
        let clipped_primitives = self
            .egui_winit_state
            .egui_ctx()
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        (
            clipped_primitives,
            full_output.textures_delta,
            available_rect,
            self.egui_winit_state.egui_ctx().pixels_per_point(),
        )
    }

    fn ui<C: ControllerTrait>(
        &self,
        ctx: &Context,
        ui_state: &UiState,
        controller: &mut C,
        event_proxy: &EventLoopProxy<CustomEvent<C>>,
    ) {
        controller.ui(ctx, ui_state, |user_event| {
            let _ = event_proxy.send_event(CustomEvent::UserEvent(user_event));
        });
    }
}
