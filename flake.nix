{
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable;

    naersk = {
      url = github:nix-community/naersk;
    };

    utils = {
      url = github:numtide/flake-utils;
    };

    lingua-franca-tests = {
      url = github:lf-lang/lingua-franca;
      flake = false;
    };

    lingua-franca-compiler = {
      url = "https://github.com/lf-lang/lingua-franca/releases/download/nightly/lf-cli-nightly-20230516-050417.tar.gz";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, naersk, utils, lingua-franca-tests, lingua-franca-compiler, ... }:
    utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          package = pkgs.callPackage ./derivation.nix {
            naersk = naersk.lib.${system};
          };

          lfc = pkgs.callPackage ./nix/lfc.nix {
            lingua-franca-compiler-source = lingua-franca-compiler;
          };

          unit-tests = pkgs.callPackage ./nix/test.nix {
            source = lingua-franca-tests;
            lingua-franca = lfc;
            lingo = package;
          };

        in
        rec {
          checks = packages;
          packages.lingo = package;
          packages.tests = unit-tests;
          defaultPackage = package;
          overlay = (final: prev: {
            lingo = package;
          });
        }
      ) // {
      hydraJobs =
        let
          hydraSystems = [
            "x86_64-linux"
            "aarch64-linux"
          ];
        in
        builtins.foldl'
          (hydraJobs: system:
            builtins.foldl'
              (hydraJobs: pkgName:
                nixpkgs.lib.recursiveUpdate hydraJobs {
                  ${pkgName}.${system} = self.packages.${system}.${pkgName};
                }
              )
              hydraJobs
              (builtins.attrNames self.packages.${system})
          )
          { }
          hydraSystems;
    };
}
