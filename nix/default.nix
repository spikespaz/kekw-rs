{ craneLib,
# Must be provided via `callPackage`.
sourceRoot ? ./..,
#
platforms ? [ "x86-64-linux" ],
#
lib, pkg-config, openssl
#
}:
let
  commonArgs = {
    strictDeps = true;
    src = craneLib.cleanCargoSource (craneLib.path sourceRoot);
    nativeBuildInputs = [ pkg-config openssl ];
    meta = {
      # inherit (manifest.package) description homepage;
      license = lib.licenses.mit;
      maintainers = [ lib.maintainers.spikespaz ];
      inherit platforms;
      # mainProgram = manifest.package.name;
    };
  };
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in craneLib.buildPackage (commonArgs // {
  inherit cargoArtifacts;
})
