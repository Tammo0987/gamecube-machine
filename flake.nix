{
  description = "NixOS module for GameCube emulator kiosk";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      apps = import ./apps { inherit pkgs; };
      module = import ./module.nix;
    in {
      nixosModules.default = module;

      lib.mkSystem = import ./lib/mkSystem.nix {
        inherit nixpkgs module apps;
      };
    };
}
