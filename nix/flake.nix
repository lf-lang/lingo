{
  description = "example package which says hello";

  inputs = {
    lf-pkgs.url = "git+https://gitea.tassilo-tanneberger.de/revol-xut/lf-pkgs.git";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-22.05";
  };

  outputs = {self, lf-pkgs, nixpkgs}: 
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      lib = pkgs.lib;
      buildLinguaFranca = lf-pkgs.lib."${system}".buildLinguaFranca;
      LFPackages = lf-pkgs.packages.${system};
    in {
      packages."${system}" = {
        default = pkgs.callPackage ./derivation.nix {
          buildLinguaFranca = buildLinguaFranca;
          lfPackages = LFPackages;
        };
      } // LFPackages;
  };
}
