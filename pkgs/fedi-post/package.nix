{
  toot,
  nixpkgs-prs,
  writeShellApplication,
}:
writeShellApplication {
  name = "nixpkgs-fedibot";
  text = builtins.readFile ./script.sh;
  runtimeInputs = [
    toot
    nixpkgs-prs
  ];
}
