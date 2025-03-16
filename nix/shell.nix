{
  mkShell,
  callPackage,

  # rtp
  toot,

  # extra tooling
  clippy,
  rustfmt,
  rust-analyzer,
}:
let
  defaultPackage = callPackage ./package.nix { };
in
mkShell {
  inputsFrom = [ defaultPackage ];

  packages = [
    toot
    clippy
    rustfmt
    rust-analyzer
  ];
}
