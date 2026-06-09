# SPDX-FileCopyrightText: 2021 Serokell <https://serokell.io/>
#
# SPDX-License-Identifier: CC0-1.0
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    { nixpkgs, flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      perSystem =
        {
          config,
          self',
          inputs',
          pkgs,
          system,
          lib,
          ...
        }:
        let
          # Common dependencies for all platforms
          commonBuildInputs = with pkgs; [
            nixfmt-rfc-style
            nixd
            bun
            pkg-config
            rustc
            cargo
            #rust-analyzer
            clippy
            openssl
            rustfmt
            wasm-pack
            wasm-bindgen-cli
            clang
            lld
            worker-build
            fontconfig
            freetype
          ];

          # Linux-specific dependencies (includes Wayland)
          linuxBuildInputs = with pkgs; [
            # Wayland dependencies for native node GUI
            wayland
            libxkbcommon
            wayland-protocols

            # Graphics libraries
            libGL
            vulkan-loader
            mesa

            # X11 fallback dependencies
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
          ];

          # macOS-specific dependencies
          darwinBuildInputs = with pkgs; [
            # macOS frameworks are handled by the system
            # Add any macOS-specific packages here if needed
            libiconv
          ];

          # Platform-specific library paths
          linuxLibraryPath = lib.makeLibraryPath (with pkgs; [
            wayland
            libxkbcommon
            vulkan-loader
            libGL
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            fontconfig
            freetype
          ]);

          # Determine platform-specific inputs
          platformBuildInputs =
            if pkgs.stdenv.isLinux then
              linuxBuildInputs
            else if pkgs.stdenv.isDarwin then
              darwinBuildInputs
            else
              [];
        in
        {
          devShells.default = pkgs.mkShell {
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            RUST_BACKTRACE = 1;

            # Platform-specific environment variables
            LD_LIBRARY_PATH = if pkgs.stdenv.isLinux then linuxLibraryPath else "";

            # macOS-specific environment variables
            DYLD_FALLBACK_LIBRARY_PATH = if pkgs.stdenv.isDarwin then
              "${pkgs.libiconv}/lib"
            else "";

            nativeBuildInputs = commonBuildInputs ++ platformBuildInputs;

            shellHook = ''
              echo "🚀 MPC Wallet Development Environment"
              echo "Platform: ${system}"
              echo ""
              if [[ "${system}" == *"linux"* ]]; then
                echo "✅ Linux environment configured with Wayland support"
              elif [[ "${system}" == *"darwin"* ]]; then
                echo "✅ macOS environment configured"
                echo "Note: Wayland dependencies are not available on macOS"
              fi
              echo ""
              echo "Available commands:"
              echo "  cargo build                                  - Build the workspace"
              echo "  cargo run --bin frost-mpc-tui -p tui-node   - Run the TUI"
              echo "  cargo run --bin frost-mpc-cli -p frost-mpc-cli -- --help  - Headless CLI"
              echo "  bun install                                  - Install JS dependencies"
              echo "  bun run build:wasm                           - Build WebAssembly modules"
              echo ""
            '';
          };
        };
    };
}