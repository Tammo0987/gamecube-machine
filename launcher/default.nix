{ pkgs, ... }:

pkgs.rustPlatform.buildRustPackage {
  pname = "gamecube-launcher";
  version = "0.1.0";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = with pkgs; [
    pkg-config
    autoPatchelfHook
    makeWrapper
  ];

  buildInputs = with pkgs; [
    stdenv.cc.cc.lib
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

  # Baked in at compile time via option_env!()
  LAUNCHER_FONT = "${pkgs.roboto}/share/fonts/truetype/Roboto-Bold.ttf";
  DOLPHIN_ICON  = "${pkgs.dolphin-emu}/share/icons/hicolor/256x256/apps/dolphin-emu.png";

  # miniquad uses dlopen() for X11/Wayland/GL — LD_LIBRARY_PATH is required at runtime
  postFixup = ''
    wrapProgram "$out/bin/gamecube-launcher" \
      --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath (with pkgs; [
        libGL
        libxkbcommon
        wayland
        xorg.libX11
        xorg.libXi
        xorg.libXcursor
        xorg.libXrandr
      ])}"
  '';
}
