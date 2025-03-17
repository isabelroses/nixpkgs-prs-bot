{
  mkShell,
  callPackage,

  # extra tooling
  clippy,
  rustfmt,
  rust-analyzer,
  git-cliff,
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
    git-cliff
  ];
}
