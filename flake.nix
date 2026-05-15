{
  description = "NixOS module for GameCube emulator kiosk";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      apps = import ./apps { inherit pkgs; };
      module = import ./module.nix;
    in {
      nixosModules.default = module;

      lib.mkSystem = import ./lib/mkSystem.nix {
        inherit nixpkgs module apps;
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
          rust-analyzer
          pkg-config
          dolphin-emu
          libGL
          libxkbcommon
          alsa-lib
          udev
          wayland
          xorg.libX11
          xorg.libXi
          xorg.libXcursor
          xorg.libXrandr
        ];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
          libGL
          libxkbcommon
          wayland
          xorg.libX11
          xorg.libXi
          xorg.libXcursor
          xorg.libXrandr
        ]);
        LAUNCHER_FONT = "${pkgs.roboto}/share/fonts/truetype/Roboto-Bold.ttf";
        DOLPHIN_ICON  = "${pkgs.dolphin-emu}/share/icons/hicolor/256x256/apps/dolphin-emu.png";
      };
    };
}
