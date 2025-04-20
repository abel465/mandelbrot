use spirv_builder::SpirvBuilder;
use std::env;
use std::path::Path;

fn build_shader(path_to_crate: &str) {
    let builder_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path_to_crate = builder_dir.join(path_to_crate);
    let mut builder = SpirvBuilder::new(path_to_crate, "spirv-unknown-vulkan1.1");
    if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "wasm32" {
        builder = builder.shader_crate_features(["emulate_constants".into()]);
    }
    builder.build().unwrap();
}

fn main() {
    build_shader("../shader/shader");
}
