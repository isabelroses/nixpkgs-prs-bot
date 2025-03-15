{
  bsky-cli,
  nixpkgs-prs,
  writeShellApplication,
}:
writeShellApplication {
  name = "nixpkgs-bskybot";
  text = builtins.readFile ./script.sh;
  runtimeInputs = [
    bsky-cli
    nixpkgs-prs
  ];
}
