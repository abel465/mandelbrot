{
  description = "mandelbrot";

  inputs = {
    fenix.url = "github:nix-community/fenix/3b89d5df39afc6ef3a8575fa92d8fa10ec68c95f";
    fenix.inputs.nixpkgs.follows = "nixpkgs";

    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    fenix,
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem = {
        pkgs,
        system,
        ...
      }: let
        rustPkg = with fenix.packages.${system};
          combine [
            targets.wasm32-unknown-unknown.latest.rust-std
            (latest.withComponents
              [
                "rust-src"
                "rustc-dev"
                "llvm-tools-preview"
                "cargo"
                "clippy"
                "rustc"
                "rustfmt"
                "rust-analyzer"
              ])
          ];
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustPkg;
          rustc = rustPkg;
        };
        buildInputs = with pkgs; [
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          vulkan-loader
          vulkan-tools
          wayland
          libxkbcommon
          libgcc.lib
        ];
        shadersCompilePath = "$HOME/.cache/rust-gpu-shaders";
        rustPackage = rustPlatform.buildRustPackage {
          pname = "mandelbrot";
          version = "0.0.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoLock.outputHashes = {
            "rustc_codegen_spirv-0.9.0" = "sha256-fVE62L+uyzB0Ydol1q1Ge+8S/oYZxZYZj4eGBsYi6S0=";
            "easy-shader-runner-0.0.0" = "sha256-iYcMFzjWS3LA8xyJZAB4TUhHxOY4/nC3aQ+OwEz24RU=";
          };
          buildNoDefaultFeatures = true;
          buildFeatures = ["runtime-compilation"];
          dontCargoSetupPostUnpack = true;
          postUnpack = ''
            mkdir -p .cargo
            cat "$cargoDeps"/.cargo/config.toml | sed "s|cargo-vendor-dir|$cargoDeps|" >> .cargo/config.toml
            # HACK(eddyb) bypass cargoSetupPostPatchHook.
            export cargoDepsCopy="$cargoDeps"
          '';
          nativeBuildInputs = [pkgs.makeWrapper];
          configurePhase = ''
            export SHADERS_TARGET_DIR=${shadersCompilePath}
          '';
          fixupPhase = ''
            cp -r . $out/repo
            wrapProgram $out/bin/runner \
              --set LD_LIBRARY_PATH $LD_LIBRARY_PATH:$out/lib:${nixpkgs.lib.makeLibraryPath buildInputs} \
              --set PATH $PATH:${nixpkgs.lib.makeBinPath [rustPkg]} \
              --set CARGO_MANIFEST_DIR $out/repo/runner
          '';
        };
      in rec {
        packages.default = pkgs.writeShellScriptBin "mandelbrot" ''
          export CARGO_TARGET_DIR="${shadersCompilePath}"
          exec -a "$0" "${rustPackage}/bin/runner" "$@"
        '';
        apps.default = {
          type = "app";
          program = "${packages.default}/bin/mandelbrot";
        };
        devShells.default = with pkgs;
          mkShell {
            nativeBuildInputs = [rustPkg wasm-pack nodejs vulkan-validation-layers spirv-tools];
            LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
          };
      };
    };
}
