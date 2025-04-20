use structopt::StructOpt;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen::{self, prelude::*};

mod controller;

#[derive(StructOpt, Clone, Copy)]
pub struct Options {
    /// Starts in debug mode and with speed set to 0
    #[structopt(short, long)]
    debug: bool,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    let options = Options::from_args();
    let controller = controller::Controller::new(&options);
    cfg_if::cfg_if! {
        if #[cfg(all(
            any(feature = "hot-reload-shader", feature = "runtime-compilation"),
            not(target_arch = "wasm32")
        ))] {
            easy_shader_runner::run_with_runtime_compilation(controller, "../shader/shader");
        } else {
            easy_shader_runner::run_with_prebuilt_shader(controller, include_bytes!(env!("shader.spv")));
        }
    }
}
