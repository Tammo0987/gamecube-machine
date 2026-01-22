{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.gamecube-emulator;
  gfx = cfg.graphics;

  dolphinConfig = pkgs.writeText "Dolphin.ini" ''
    [General]
    ISOPath0 = ${cfg.gamesDirectory}
    ISOPaths = 1
    ShowLag = False
    ShowFrameCount = False

    [Core]
    SIDevice0 = 6
    SIDevice1 = 0
    SIDevice2 = 0
    SIDevice3 = 0

    [Display]
    RenderToMain = False
    Fullscreen = True
    DisableScreenSaver = True
    KeepWindowOnTop = True
    RenderWindowAutoSize = False

    [Interface]
    ConfirmStop = False
    CursorVisibility = 2
    ShowActiveTitle = False
    UseBuiltinTitleDatabase = True
    ThemeName = ${cfg.theme}
    OnScreenDisplayMessages = False

    [Analytics]
    Enabled = False
    PermissionAsked = True

    [Input]
    BackgroundInput = True
  '';

  dolphinGfxConfig = pkgs.writeText "GFX.ini" ''
    [Settings]
    ShowFPS = ${lib.boolToString gfx.showFPS}
    AspectRatio = ${toString gfx.aspectRatio}
    InternalResolution = ${toString gfx.internalResolution}
    MSAA = ${toString gfx.msaa}
    WaitForShadersBeforeStarting = ${lib.boolToString gfx.waitForShaders}

    [Hardware]
    VSync = ${lib.boolToString gfx.vsync}

    [Hacks]
    SkipDuplicateXFBs = True
    XFBToTextureEnable = True
  '';

  dolphinGCPadConfig = pkgs.writeText "GCPadNew.ini" ''
    [GCPad1]
    Device = SDL/1/Nintendo Switch Pro Controller
    Buttons/A = `Button E`
    Buttons/B = `Button S`
    Buttons/X = `Button W`
    Buttons/Y = `Button N`
    Buttons/Z = `Trigger R`
    Buttons/Start = `Start`
    Main Stick/Up = `Left Y+`
    Main Stick/Down = `Left Y-`
    Main Stick/Left = `Left X-`
    Main Stick/Right = `Left X+`
    Main Stick/Calibration = 100.00 141.42 100.00 141.42 100.00 141.42 100.00 141.42
    C-Stick/Up = `Right Y+`
    C-Stick/Down = `Right Y-`
    C-Stick/Left = `Right X-`
    C-Stick/Right = `Right X+`
    C-Stick/Calibration = 100.00 141.42 100.00 141.42 100.00 141.42 100.00 141.42
    Triggers/L = `Shoulder L`
    Triggers/R = `Shoulder R`
    Triggers/L-Analog = `Trigger L`
    Triggers/R-Analog = `Trigger R`
    D-Pad/Up = `Pad N`
    D-Pad/Down = `Pad S`
    D-Pad/Left = `Pad W`
    D-Pad/Right = `Pad E`
  '';

in
{
  options.services.gamecube-emulator = {
    gamesDirectory = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/dolphin-emu/games";
      description = "Directory where game ISOs are stored";
    };

    theme = lib.mkOption {
      type = lib.types.str;
      default = "Clean Emerald";
      description = "Dolphin UI theme";
    };

    graphics = {
      internalResolution = lib.mkOption {
        type = lib.types.int;
        default = 2;
        description = "Internal resolution multiplier (1 = native, 2 = 2x, 4 = 4x, etc.)";
      };

      msaa = lib.mkOption {
        type = lib.types.enum [
          0
          2
          4
          8
        ];
        default = 2;
        description = "Anti-aliasing samples (0 = off, 2 = 2x, 4 = 4x, 8 = 8x)";
      };

      vsync = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Enable vertical sync";
      };

      showFPS = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Show FPS counter";
      };

      aspectRatio = lib.mkOption {
        type = lib.types.enum [
          0
          1
          2
          3
        ];
        default = 0;
        description = "Aspect ratio (0 = auto, 1 = 16:9, 2 = 4:3, 3 = stretch)";
      };

      waitForShaders = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Wait for shaders to compile before starting";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    # Dolphin config files exposed for kiosk script
    services.gamecube-emulator._dolphinConfigs = {
      inherit dolphinConfig dolphinGfxConfig dolphinGCPadConfig;
    };

    # Packages
    environment.systemPackages = [ pkgs.dolphin-emu ];

    # Udev rules for Dolphin
    services.udev.packages = [ pkgs.dolphin-emu ];
  };
}
