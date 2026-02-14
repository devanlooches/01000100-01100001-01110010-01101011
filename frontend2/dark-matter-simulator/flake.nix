{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
    }:
    let
      pkgs = import nixpkgs {
        system = "aarch64-darwin";
      };

      # Read the file relative to the flake's root
      overrides = {
        toolchain = {
          channel = "stable";
        };
      };
      libPath =
        with pkgs;
        lib.makeLibraryPath [
          # load external libraries that you need in your rust project here
        ];
    in
    {
      devShells.aarch64-darwin.default = pkgs.mkShell rec {
        buildInputs = with pkgs; [
          pkg-config
          openssl
          libsodium.dev
          libsodium.out
        ];
        nativeBuildInputs = with pkgs; [
          cargo
          cargo-edit
          cargo-leptos
          rustc
          (pkgs.callPackage buildWasmBindgenCli rec {
            src = fetchCrate {
              pname = "wasm-bindgen-cli";
              version = "0.2.106";
              hash = "sha256-M6WuGl7EruNopHZbqBpucu4RWz44/MSdv6f0zkYw+44=";
            };

            cargoDeps = rustPlatform.fetchCargoVendor {
              inherit src;
              inherit (src) pname version;
              hash = "sha256-ElDatyOwdKwHg3bNH/1pcxKI7LXkhsotlDPQjiLHBwA=";
            };
          })
          rustfmt
          clippy
          rust-analyzer
          tailwindcss_4
          llvmPackages.libclang
          llvmPackages.clang-unwrapped
          lld
          leptosfmt
        ];
        shellHook = ''
          export EDITOR=hx
          export CC_wasm32_unknown_unknown=${pkgs.llvmPackages.clang-unwrapped}/bin/clang-21
          export CFLAGS_wasm32_unknown_unknown="-I ${pkgs.llvmPackages.libclang.lib}/lib/clang/21/include/"
          export PATH="/opt/homebrew/opt/llvm/bin/:$PATH"
          export CC=${pkgs.llvmPackages.clang}/bin/clang
          export AR=${pkgs.llvmPackages.bintools-unwrapped}/bin/llvm-ar
          # zellij --layout layout.kdl
        '';
      };
    };
}
