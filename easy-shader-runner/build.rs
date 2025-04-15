fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // While OUT_DIR is set for both build.rs and compiling the crate, PROFILE is only set in
    // build.rs. So, export it to crate compilation as well.
    let profile = std::env::var("PROFILE").unwrap();
    println!("cargo:rustc-env=PROFILE={profile}");
}
