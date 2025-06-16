# mandelbrot
Cool fractal

## Try it out
```bash
nix run github:abel465/mandelbrot
```

## Set up
```bash
git clone https://github.com/abel465/mandelbrot.git
cd mandelbrot/
nix develop
```

## Run
### Native
```bash
cargo run
```

### Wasm
```bash
cd wasm-app
npm install
npm run wasm-pack-dev
npm run dev
```
