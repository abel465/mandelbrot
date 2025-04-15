use egui_winit::winit::event_loop::EventLoop;
use std::borrow::Cow;
use user_event::CustomEvent;

pub use bind_group_buffer::BufferDescriptor;
pub use controller::ControllerTrait;
pub use ui::UiState;
pub use user_event::UserEvent;

pub use egui_wgpu::wgpu;
pub use egui_winit::egui;
pub use egui_winit::winit;

mod app;
mod bind_group_buffer;
mod context;
mod controller;
mod fps_counter;
mod render_pass;
#[cfg(all(
    any(feature = "runtime-compilation", feature = "hot-reload-shader"),
    not(target_arch = "wasm32")
))]
mod shader;
mod ui;
mod user_event;

const TITLE: &str = "runner";

#[cfg(all(
    any(feature = "runtime-compilation", feature = "hot-reload-shader"),
    not(target_arch = "wasm32")
))]
pub fn run_with_runtime_compilation<C: ControllerTrait + Send>(
    controller: C,
    shader_crate_path: impl AsRef<std::path::Path>,
) {
    setup_logging();
    let event_loop = EventLoop::with_user_event().build().unwrap();
    // Build the shader before we pop open a window, since it might take a while.
    let shader_path = shader::compile_shader(
        #[cfg(feature = "hot-reload-shader")]
        event_loop.create_proxy(),
        shader_crate_path,
    );
    let shader_bytes = std::fs::read(shader_path).unwrap();
    start(event_loop, controller, shader_bytes)
}

pub fn run_with_prebuilt_shader<C: ControllerTrait>(controller: C, shader_bytes: &'static [u8]) {
    setup_logging();
    let event_loop = EventLoop::with_user_event().build().unwrap();
    start(event_loop, controller, shader_bytes);
}

fn start<C: ControllerTrait>(
    event_loop: EventLoop<CustomEvent<C>>,
    controller: C,
    shader_bytes: impl Into<Cow<'static, [u8]>>,
) {
    let mut app = app::App::new(event_loop.create_proxy(), shader_bytes.into(), controller);
    event_loop.run_app(&mut app).unwrap()
}

fn setup_logging() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            let _ = console_log::init();
        } else {
            let mut rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());
            for loud_crate in ["naga", "wgpu_core", "wgpu_hal"] {
                if !rust_log.contains(&format!("{loud_crate}=")) {
                    rust_log += &format!(",{loud_crate}=warn");
                }
            }
            std::env::set_var("RUST_LOG", rust_log);
            let _ = env_logger::try_init();
        }
    }
}
