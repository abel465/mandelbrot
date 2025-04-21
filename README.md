# mandelbrot
Cool fractal

## Try with nix
```bash
nix run github:abel465/mandelbrot
```

## Set up development environment
```bash
git clone https://github.com/abel465/mandelbrot.git
cd mandelbrot/
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
npm install
wasm-pack build ../runner --out-dir ../wasm-app/pkg --dev
npm run dev
```
