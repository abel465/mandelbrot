use crate::bind_group_buffer::BufferDescriptor;
use crate::user_event::UserEvent;
use egui_winit::winit::event::{ElementState, KeyEvent, MouseButton};
use glam::*;

pub trait ControllerTrait: 'static {
    fn resize(&mut self, _size: UVec2);

    fn mouse_move(&mut self, _position: Vec2) {}

    fn mouse_scroll(&mut self, _delta: Vec2) {}

    fn mouse_input(&mut self, _state: ElementState, _button: MouseButton) {}

    fn keyboard_input(&mut self, _key: KeyEvent) {}

    fn prepare_render(&mut self, offset: Vec2) -> impl bytemuck::NoUninit;

    fn update<F: Fn(UVec2, &[u8])>(&mut self, _compute: F, _allowed_duration: f32) {}

    fn buffers(&self) -> Vec<BufferDescriptor> {
        vec![]
    }

    fn ui<F: Fn(UserEvent)>(
        &mut self,
        _ctx: &egui::Context,
        _ui_state: &crate::ui::UiState,
        _send_event: F,
    ) {
    }
}
