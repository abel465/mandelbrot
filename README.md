# mandelbrot
Cool fractal

## Try with nix
```bash
nix run github:abel465/mandelbrot
```

## Set up development environment
```bash
git clone https://github.com/abel465/mandelbrot.git
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
wasm-pack build ../mandelbrot --out-dir ../../wasm-app/pkg --dev
npm install
npm run dev
```
