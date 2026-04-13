{
  description = "Development environment for ALSA-based projects";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          # Tools needed at build-time (like compilers and pkg-config)
          nativeBuildInputs = with pkgs; [
            pkg-config
            gcc
            gnumake
          ];

          # Libraries needed at link-time
          buildInputs = with pkgs; [
            alsa-lib
          ];

          shellHook = ''
            echo "ALSA development environment loaded."
            echo "alsa-lib version: $(pkg-config --modversion alsa)"
          '';
        };
      }
    );
}
