# easy-shader-runner
Use rust shaders and egui on the web and native

## How to use
Implement `easy_shader_runner::ControllerTrait` and call `easy_shader_runner::run*`

## Try with nix
```bash
nix run github:abel465/easy-shader-runner
```

## Set up development environment
```bash
git clone https://github.com/abel465/easy-shader-runner.git
cd easy-shader-runner/
nix develop
```

## Run the example
### Native
```bash
cargo run
```

### Wasm
```bash
cd wasm-app
wasm-pack build ../example --out-dir ../../wasm-app/pkg --dev
npm install
npm run dev
```
