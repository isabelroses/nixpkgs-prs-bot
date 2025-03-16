{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;

      forAllSystems =
        function: lib.genAttrs lib.systems.flakeExposed (system: function nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (pkgs: {
        default = self.packages.${pkgs.stdenv.hostPlatform.system}.nixpkgs-prs;
        nixpkgs-prs = pkgs.callPackage ./nix/package.nix { };
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.callPackage ./nix/shell.nix { };
      });

      nixosModules = {
        default = self.nixosModules.nixpkgs-prs-bot;

        nixpkgs-prs-bot = {
          _file = "${self.outPath}/flake.nix#$nixosModules.nixpkgs-prs-bot";
          imports = [ ./nix/module.nix ];
        };
      };
    };
}
