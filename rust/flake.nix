{
  description = "hecto-study development environment";

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
        buildInputs = with pkgs; [cargo rustfmt];
        shellHook = ''
          echo "cargo"
          which cargo
          cargo --version
        '';
      };
    });

    packages = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
      };
    in {
      default = pkgs.stdenv.mkDerivation {
        name = "hecto-study";
        src = ./.;
        buildInputs = with pkgs; [cargo];
        buildPhase = ''
          cargo build
        '';
        installPhase = ''
          install -D -t $out/bin target/debug/hecto
        '';
      };
    });

    apps = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
      };
    in {
      default = {
        type = "app";
        buildInputs = with pkgs; [cargo];
        program = toString (pkgs.writeShellScript "cargo-run" ''
          cargo run -- "$@"
        '');
      };
      cargo = {
        type = "app";
        buildInputs = with pkgs; [cargo cargo-clippy];
        program = toString (pkgs.writeShellScript "cargo-run" ''
          cargo "$@"
        '');
      };
      test = {
        type = "app";
        buildInputs = with pkgs; [cargo cargo-clippy];
        program = toString (pkgs.writeShellScript "cargo-run" ''
          cargo test
        '');
      };
      clippy = {
        type = "app";
        buildInputs = with pkgs; [cargo cargo-clippy];
        program = toString (pkgs.writeShellScript "cargo-run" ''
          cargo clippy
        '');
      };
    });
  };
}
