{
  jq,
  gh,
  writeShellApplication,
}:
writeShellApplication {
  name = "nixpkgs-prs";
  text = builtins.readFile ./script.sh;
  runtimeInputs = [
    jq
    gh
  ];
}
