{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems = {
      url = "github:nix-systems/default-linux";
      flake = false;
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    nixfmt.url = "github:serokell/nixfmt/v0.6.0";
  };

  outputs =
    { self, nixpkgs, systems, rust-overlay, crane, advisory-db, nixfmt }:
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

      rustNightly = rust-bin:
        (rust-bin.selectLatestNightlyWith (toolchain:
          toolchain.minimal.override {
            extensions = [ "rust-docs" "rustfmt" "rust-analyzer" ];
          }));

      intermediates = pkgs:
        with pkgs; rec {
          commonArgs = {
            strictDeps = true;
            src = craneLib.cleanCargoSource (craneLib.path self.outPath);
            nativeBuildInputs = [ pkg-config openssl ];
            meta = {
              license = lib.licenses.mit;
              maintainers = [ lib.maintainers.spikespaz ];
              platforms = import systems;
            };
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        };
    in {
      devShells = eachSystem (system:
        with pkgsFor.${system}; {
          default = craneLib.devShell {
            strictDeps = true;
            inputsFrom = [ kekw-bot ];
            packages = [
              # Derivations in `rustToolchain` take precedence over nightly.
              # Use rustfmt, and other tools that require nightly features.
              (rustNightly rust-bin)
              cargo-audit
              cargo-watch
              cargo-nextest
            ];
            OPENSSL_LIB_DIR = "${openssl.out}/lib";
            OPENSSL_ROOT_DIR = "${openssl.out}";
            OPENSSL_INCLUDE_DIR = "${openssl.dev}/include";
            # RUST_BACKTRACE = 1;
          };
        });

      overlays = {
        default = final: _:
          let inherit (intermediates final) commonArgs cargoArtifacts;
          in {
            kekw-bot = final.craneLib.buildPackage (commonArgs // {
              doCheck = false; # handled by nextest
              inherit cargoArtifacts;
            });
          };
      };

      packages = eachSystem (system: {
        default = self.packages.${system}.kekw-bot;
        inherit (pkgsFor.${system}) kekw-bot;
      });

      checks = eachSystem (system:
        with pkgsFor.${system};
        let inherit (intermediates pkgs) commonArgs cargoArtifacts;
        in {
          inherit kekw-bot;

          kekw-rs-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          kekw-rs-doc =
            craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

          # Check formatting
          kekw-rs-fmt = let
            craneLib =
              (crane.mkLib pkgs).overrideToolchain (rustNightly rust-bin);
          in craneLib.cargoFmt { inherit (commonArgs) src; };

          # Audit dependencies
          kekw-rs-audit = craneLib.cargoAudit {
            inherit (commonArgs) src;
            inherit advisory-db;
          };

          # Audit licenses
          # kekw-rs-deny = craneLib.cargoDeny { inherit (commonArgs) src; };

          kekw-rs-nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
        });

      formatter = eachSystem (system: nixfmt.packages.${system}.default);
    };
}
