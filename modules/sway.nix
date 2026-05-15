{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.gamecube-emulator;
  dolphinConfigs = cfg._dolphinConfigs;
  launcher = pkgs.callPackage ../launcher { };

  swayConfig = pkgs.writeText "sway-kiosk-config" ''
    output * bg #000000 solid_color

    # Hide cursor after 1 second of inactivity
    seat * hide_cursor 1000

    exec ${launcher}/bin/gamecube-launcher --games-dir ${cfg.gamesDirectory}

    # Dolphin main window - fullscreen
    for_window [app_id="dolphin-emu"] fullscreen enable

    # Game render window (has | in title) - fullscreen and grab focus
    for_window [title=".*\|.*"] fullscreen enable, focus

    # Config dialogs float on top
    for_window [title="^(Configure|Controller|Graphics|Hotkey|About|Settings|Properties|Verify|Add|Edit).*"] {
      fullscreen disable
      floating enable
    }

    default_border none
    default_floating_border none

    gaps inner 0
    gaps outer 0

    focus_follows_mouse no
    focus_on_window_activation focus

    bindsym Mod4+Shift+e exec swaymsg exit
  '';

  kioskScript = pkgs.writeShellScript "dolphin-kiosk" ''
    export XDG_RUNTIME_DIR=/run/user/$(id -u)
    export QT_QPA_PLATFORM=wayland
    export XDG_SESSION_TYPE=wayland
    export WLR_NO_HARDWARE_CURSORS=1

    mkdir -p ~/.config/sway
    cp ${swayConfig} ~/.config/sway/config

    # Setup Dolphin config
    mkdir -p ~/.config/dolphin-emu
    cp -f ${dolphinConfigs.dolphinConfig} ~/.config/dolphin-emu/Dolphin.ini
    cp -f ${dolphinConfigs.dolphinGfxConfig} ~/.config/dolphin-emu/GFX.ini

    exec ${pkgs.sway}/bin/sway
  '';

in
{
  options.services.gamecube-emulator = {
    _dolphinConfigs = lib.mkOption {
      type = lib.types.attrs;
      internal = true;
      description = "Internal option to pass Dolphin config files between modules";
    };
  };

  config = lib.mkIf cfg.enable {
    # Kiosk service
    systemd.services.dolphin-kiosk = {
      description = "Dolphin Emulator Kiosk";
      wantedBy = [ "multi-user.target" ];
      after = [
        "systemd-user-sessions.service"
        "systemd-logind.service"
      ];
      wants = [
        "dbus.socket"
        "systemd-logind.service"
      ];
      conflicts = [ "getty@tty1.service" ];

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        TTYPath = "/dev/tty1";
        TTYReset = "yes";
        TTYVHangup = "yes";
        TTYVTDisallocate = "yes";
        StandardInput = "tty-fail";
        StandardOutput = "journal";
        StandardError = "journal";
        PAMName = "login";
        WorkingDirectory = "/home/${cfg.user}";
        ExecStart = kioskScript;
      };
    };

    # Packages
    environment.systemPackages = with pkgs; [
      sway
      xdg-utils
      qt6.qtwayland
    ];

    # Graphics & input
    hardware.graphics.enable = true;
    services.libinput.enable = true;
    security.polkit.enable = true;

    # Audio
    security.rtkit.enable = true;
    services.pipewire = {
      enable = true;
      alsa.enable = true;
      alsa.support32Bit = true;
      pulse.enable = true;
    };

    # Wayland portal
    xdg.portal = {
      enable = true;
      wlr.enable = true;
      extraPortals = [ pkgs.xdg-desktop-portal-gtk ];
      config.common.default = [
        "wlr"
        "gtk"
      ];
    };
  };
}
