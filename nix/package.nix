{
  lib,
  rustPlatform,
  openssl,
  pkg-config,
  makeBinaryWrapper,
  toot,
}:
let
  toml = (lib.importTOML ../Cargo.toml).workspace.package;

  rtp = lib.makeBinPath [
    toot
  ];
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
        ../crates
      ]
    );
  };

  cargoBuildFlags = [
    "--package"
    "nixpkgs-prs"
  ];

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    makeBinaryWrapper
  ];

  buildInputs = [
    openssl
  ];

  postFixup = ''
    wrapProgram $out/bin/nixpkgs-prs \
      --prefix PATH : ${rtp}
  '';

  meta = {
    inherit (toml) homepage description;
    license = lib.licenses.eupl12;
    maintainers = with lib.maintainers; [ isabelroses ];
    mainProgram = "nixpkgs-prs";
  };
}
