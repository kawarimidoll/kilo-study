{
  description = "kilo-study development environment";

  inputs.nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

  outputs = {
    self,
    nixpkgs,
  }: let
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
  in {
    devShells = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
      };
    in {
      default = pkgs.mkShell {
        buildInputs = with pkgs; [gcc gnumake];
        shellHook = ''
          echo "cc"
          which cc
          cc --version
          echo "make"
          which make
          make --version
        '';
      };
    });

    packages = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
      };
    in {
      default = pkgs.stdenv.mkDerivation {
        name = "kilo-study";
        src = ./.;
        buildInputs = with pkgs; [gcc];
        buildPhase = ''
          gcc kilo.c -o kilo -Wall -Wextra -pedantic -std=c99
        '';
        installPhase = ''
          install -D -t $out/bin kilo
        '';
      };
    });
  };
}
