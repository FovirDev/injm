{
  description = "A CLI tool that injects content into marked regions in source files.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        parserVersion = "1.12.4";
        parserSources = pkgs.fetchurl {
          url = "https://github.com/xberg-io/tree-sitter-language-pack/releases/download/v${parserVersion}/parser-sources-${parserVersion}.tar.zst";
          hash = "sha256-NBR3lkTeZMGLmOPNGj5xvmpass+uQFp7hbFgakRePUU=";
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "injm";
          version = "1.0.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          preBuild = ''
            export TSLP_SOURCE_BUNDLE_URL="file://${parserSources}"
          '';
          doCheck = false;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            clippy
            just
            markdown-toc
            pre-commit
            prettier
            rust-analyzer
            rustc
            rustfmt
          ];
        };
      }
    );
}
