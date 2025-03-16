{
  mkShell,
  callPackage,

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
    clippy
    rustfmt
    rust-analyzer
  ];
}
