{
  description = "COSMIC Radio Applet - Internet radio player for COSMIC Desktop";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Runtime dependencies
        runtimeDeps = with pkgs; [
          mpv
          alsa-utils
        ];

        # Build dependencies for libcosmic/wayland
        buildInputs = with pkgs; [
          openssl
          libxkbcommon
          wayland
          alsa-lib
          fontconfig
          freetype
          libGL
          wayland-protocols
          libinput
          mesa
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
          rustToolchain
          makeWrapper
          cmake
        ];

        # Library paths for runtime
        libPath = pkgs.lib.makeLibraryPath buildInputs;

      in {
        packages = {
          default = self.packages.${system}.cosmic-radio-applet;

          cosmic-radio-applet = pkgs.rustPlatform.buildRustPackage rec {
            pname = "cosmic-radio-applet";
            version = "0.1.0";

            src = pkgs.lib.cleanSource ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            inherit nativeBuildInputs buildInputs;

            # Environment variables for build
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
            BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.lib.getVersion pkgs.clang}/include";

            # For wayland scanner
            WAYLAND_PROTOCOLS = "${pkgs.wayland-protocols}/share/wayland-protocols";

            postInstall = ''
              # Install desktop file
              install -Dm644 resources/app.desktop $out/share/applications/com.marcos.RadioApplet.desktop

              # Install metainfo
              install -Dm644 resources/app.metainfo.xml $out/share/appdata/com.marcos.RadioApplet.metainfo.xml

              # Install icon
              install -Dm644 resources/icon.svg $out/share/icons/hicolor/scalable/apps/com.marcos.RadioApplet.svg

              # Wrap binary with runtime dependencies
              wrapProgram $out/bin/cosmic-radio-applet \
                --prefix PATH : ${pkgs.lib.makeBinPath runtimeDeps} \
                --prefix LD_LIBRARY_PATH : ${libPath}
            '';

            meta = with pkgs.lib; {
              description = "Internet radio player applet for COSMIC Desktop";
              homepage = "https://github.com/marcossl10/cosmic-radio-applet";
              license = licenses.mit;
              maintainers = [ ];
              platforms = platforms.linux;
              mainProgram = "cosmic-radio-applet";
            };
          };
        };

        devShells.default = pkgs.mkShell {
          inherit buildInputs;

          nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
            # Development tools
            rust-analyzer
            clippy
            rustfmt

            # Nix tools
            nixd
            statix
            deadnix
          ]);

          # Runtime deps available in dev shell
          packages = runtimeDeps;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          LD_LIBRARY_PATH = libPath;

          shellHook = ''
            echo "COSMIC Radio Applet development environment"
            echo "Run 'cargo build' to build, 'cargo run' to test"
          '';
        };
      }
    ) // {
      # NixOS module
      nixosModules = {
        default = self.nixosModules.cosmic-radio-applet;
        cosmic-radio-applet = import ./nix/nixos-module.nix self;
      };

      # Home-manager module
      homeManagerModules = {
        default = self.homeManagerModules.cosmic-radio-applet;
        cosmic-radio-applet = import ./nix/hm-module.nix self;
      };
    };
}
