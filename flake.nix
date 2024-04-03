{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default-linux";
    rust-overlay.url = "github:oxalica/rust-overlay";
    templates.url = "github:spikespaz/flake-templates";
    nixfmt.url = "github:serokell/nixfmt/v0.6.0";
  };

  outputs = { self, nixpkgs, systems, rust-overlay, templates, nixfmt }:
    let
      lib = nixpkgs.lib.extend templates.lib.overlay;
      eachSystem = lib.genAttrs (import systems);
      pkgsFor = eachSystem (system:
        import nixpkgs {
          localSystem = system;
          overlays = [ rust-overlay.overlays.default self.overlays.default ];
        });
    in {
      devShells = eachSystem (system:
        let pkgs = pkgsFor.${system};
        in {
          default = with pkgs;
            mkShell {
              strictDeps = true;
              inputsFrom = [ kekw ];
              packages = [
                # Derivations in `rust-stable` take precedence over nightly.
                (lib.hiPrio (rust-bin.stable.latest.minimal.override {
                  extensions = [ "rust-src" "rust-docs" "clippy" ];
                }))
                # Use rustfmt, and other tools that require nightly features.
                (rust-bin.selectLatestNightlyWith (toolchain:
                  toolchain.minimal.override {
                    extensions = [ "rustfmt" "rust-analyzer" ];
                  }))
              ];

              OPENSSL_LIB_DIR = "${openssl.out}/lib";
              OPENSSL_ROOT_DIR = "${openssl.out}";
              OPENSSL_INCLUDE_DIR = "${openssl.dev}/include";
              # RUST_BACKTRACE = 1;
            };
        });

      overlays = {
        default = pkgs: _: {
          kekw = pkgs.callPackage (import ./nix/default.nix) {
            inherit lib;
            sourceRoot = self;
            sourceFilter = with lib.sources;
              cleanSourceWith {
                name = "kekw";
                src = self;
                filter =
                  mkSourceFilter self [ defaultSourceFilter rustSourceFilter ];
              };
            platforms = import systems;
          };
        };
      };

      packages = eachSystem (system: {
        default = self.packages.${system}.kekw;
        kekw = (self.overlays.default pkgsFor.${system} null).kekw;
      });

      formatter = eachSystem (system: nixfmt.packages.${system}.default);
    };
}
