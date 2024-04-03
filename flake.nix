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

      intermediates = pkgs: craneLib: rec {
        commonArgs = {
          strictDeps = true;
          src = craneLib.cleanCargoSource (craneLib.path self.outPath);
          nativeBuildInputs = with pkgs; [ pkg-config openssl ];
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

      checks = eachSystem (system:
        with pkgsFor.${system};
        let inherit (intermediates pkgs craneLib) commonArgs cargoArtifacts;
        in {
          # Build the crate as part of `nix flake check` for convenience
          inherit kekw-bot;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          kekw-rs-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          kekw-rs-doc =
            craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

          # Check formatting
          kekw-rs-fmt = craneLib.cargoFmt { inherit (commonArgs) src; };

          # Audit dependencies
          kekw-rs-audit = craneLib.cargoAudit {
            inherit (commonArgs) src;
            inherit advisory-db;
          };

          # Audit licenses
          my-crate-deny = craneLib.cargoDeny { inherit (commonArgs) src; };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `my-crate` if you do not want
          # the tests to run twice
          kekw-rs-nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
        });

      formatter = eachSystem (system: nixfmt.packages.${system}.default);
    };
}
