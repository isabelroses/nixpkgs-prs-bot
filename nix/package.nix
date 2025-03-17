{
  lib,
  rustPlatform,
  openssl,
  pkg-config,
  versionCheckHook,
}:
let
  toml = (lib.importTOML ../Cargo.toml).package;
in
rustPlatform.buildRustPackage {
  pname = "nixpkgs-prs";
  inherit (toml) version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.intersection (lib.fileset.fromSource (lib.sources.cleanSource ../.)) (
      lib.fileset.unions [
        ../Cargo.toml
        ../Cargo.lock
        ../src
      ]
    );
  };

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    versionCheckHook
  ];

  buildInputs = [
    openssl
  ];

  doInstallCheck = true;
  versionCheckProgram = "${placeholder "out"}/bin/nixpkgs-prs";
  versionCheckProgramArg = [ "--version" ];

  meta = {
    inherit (toml) homepage description;
    license = lib.licenses.eupl12;
    maintainers = with lib.maintainers; [ isabelroses ];
    mainProgram = "nixpkgs-prs";
  };
}
