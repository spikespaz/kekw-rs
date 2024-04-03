{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default-linux";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    nixfmt.url = "github:serokell/nixfmt/v0.6.0";
  };

  outputs = { self, nixpkgs, systems, rust-overlay, crane, nixfmt }:
    let
      inherit (nixpkgs) lib;
      eachSystem = lib.genAttrs (import systems);

      pkgsFor = eachSystem (system:
        import nixpkgs {
          localSystem = system;
          overlays = [
            rust-overlay.overlays.default
            self.overlays.default
            (pkgs: _: {
              craneLib = (crane.mkLib pkgs).overrideToolchain
                (rustToolchain pkgs.rust-bin);
            })
          ];
        });

      rustToolchain = rust-bin:
        rust-bin.stable.latest.minimal.override {
          extensions = [ "rust-src" "rust-docs" "clippy" ];
        };
    in {
      devShells = eachSystem (system:
        let pkgs = pkgsFor.${system};
        in {
          default = with pkgs;
            craneLib.devShell {
              strictDeps = true;
              inputsFrom = [ kekw-bot ];

              packages = [
                # Derivations in `rustStable` take precedence over nightly.
                # (lib.hiPrio (rustToolchain rust-bin))
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
          kekw-bot = pkgs.callPackage (import ./nix/default.nix) {
            inherit (pkgs) craneLib;
            sourceRoot = self.outPath;
            platforms = import systems;
          };
        };
      };

      packages = eachSystem (system: {
        default = self.packages.${system}.kekw-bot;
        inherit (pkgsFor.${system}) kekw-bot;
      });

      formatter = eachSystem (system: nixfmt.packages.${system}.default);
    };
}
