{
# Must be provided via `callPackage`.
sourceRoot ? ./..,
# Can be removed from the flake, but stays for example.
sourceFilter ? lib.cleanSourceFilter,
#
platforms ? [ "x86-64-linux" ],
#
lib, rustPlatform, coreutils, pkg-config, openssl
#
}:
let manifest = lib.importTOML "${sourceRoot}/Cargo.toml";
in rustPlatform.buildRustPackage {
  pname = manifest.package.name;
  version = manifest.package.version;
  src = lib.cleanSourceWith {
    src = sourceRoot;
    filter = sourceFilter;
  };
  cargoLock.lockFile = "${sourceRoot}/Cargo.lock";
  nativeBuildInputs = [ # Native transitive dependencies for Cargo
    pkg-config
    openssl
  ];
  meta = {
    inherit (manifest.package) description homepage;
    license = lib.licenses.mit;
    maintainers = [ lib.maintainers.spikespaz ];
    inherit platforms;
    mainProgram = manifest.package.name;
  };
}
