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
      # Custom Bazel 5.3.0 derivation for aarch64-darwin
      bazel_5_3_0 = pkgs.stdenv.mkDerivation {
        pname = "bazel";
        version = "5.3.0";

        src = pkgs.fetchurl {
          url = "https://github.com/bazelbuild/bazel/releases/download/5.3.0/bazel-5.3.0-darwin-arm64";
          sha256 = "sha256-C7hwzftvYlbBSdroUkV912C4af2UvV3NfThxf8cxJxU=";
        };

        dontUnpack = true;
        noDist = true;

        installPhase = ''
          mkdir -p $out/bin
          install -m755 $src $out/bin/bazel
        '';
      };
      libPath =
        with pkgs;
        lib.makeLibraryPath [
          # load external libraries that you need in your rust project here
          curl
        ];
    in
    {
      devShells.aarch64-darwin.default = pkgs.mkShell rec {
        buildInputs = with pkgs; [
          pkg-config
          openssl
          libsodium.dev
          libsodium.out
          curl
          zlib
          bzip2
        ];
        nativeBuildInputs = with pkgs; [
          protobuf
          cargo
          cargo-edit
          cargo-leptos
          rustc
          bazel_5_3_0
          clang
          (pkgs.callPackage buildWasmBindgenCli rec {
            src = fetchCrate {
              pname = "wasm-bindgen-cli";
              version = "0.2.108";
              hash = "sha256-UsuxILm1G6PkmVw0I/JF12CRltAfCJQFOaT4hFwvR8E=";
            };

            cargoDeps = rustPlatform.fetchCargoVendor {
              inherit src;
              inherit (src) pname version;
              hash = "sha256-iqQiWbsKlLBiJFeqIYiXo3cqxGLSjNM8SOWXGM9u43E=";
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
          binaryen
          # Python with specific versions for TensorFlow/ONNX compatibility
          # These versions (TF 1.14.0, ONNX 1.5.0, onnx-tf 1.3.0) are known to work together
          (python313.withPackages (
            ps: with ps; [
              tensorflow
              onnxruntime
              onnx
              numpy
              pip
            ]
          ))
        ];
        shellHook = ''
          export EDITOR=hx
          export CC_wasm32_unknown_unknown=${pkgs.llvmPackages.clang-unwrapped}/bin/clang-21
          export CFLAGS_wasm32_unknown_unknown="-I ${pkgs.llvmPackages.libclang.lib}/lib/clang/21/include/"
          export PATH="/opt/homebrew/opt/llvm/bin/:$PATH"
          export CC=${pkgs.llvmPackages.clang}/bin/clang
          export AR=${pkgs.llvmPackages.bintools-unwrapped}/bin/llvm-ar
          # Library paths for tensorflow-sys linking
          export LD_LIBRARY_PATH="${libPath}:$LD_LIBRARY_PATH"
          export DYLD_LIBRARY_PATH="${libPath}:$DYLD_LIBRARY_PATH"
          export PKG_CONFIG_PATH="${pkgs.curl}/lib/pkgconfig:$PKG_CONFIG_PATH"
          # Tell TensorFlow to use pre-built binaries instead of building from source
          export TF_SYSTEM_LIBS="com_google_absl,com_google_protobuf,com_googlesource_code_re2,org_sqlite"
          export BAZEL_CXXOPTS="-std=c++17"
          # zellij --layout layout.kdl
        '';
      };
    };
}
