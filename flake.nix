{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;

      forAllSystems =
        function:
        nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed (
          system: function nixpkgs.legacyPackages.${system}
        );
    in
    {
      packages = forAllSystems (
        pkgs:
        lib.packagesFromDirectoryRecursive {
          directory = ./pkgs;
          callPackage = lib.callPackageWith (pkgs // self.packages.${pkgs.stdenv.hostPlatform.system});
        }
      );

      nixosModules = {
        default = self.nixosModules.nixpkgs-prs-bot;
        nixpkgs-prs-bot = import ./module self;
      };
    };
}
