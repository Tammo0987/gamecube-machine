{
  config,
  lib,
  ...
}:

let
  cfg = config.services.gamecube-emulator;

in
{
  imports = [
    ./modules/dolphin.nix
    ./modules/launcher.nix
    ./modules/sway.nix
    ./modules/controller.nix
    ./modules/bluetooth.nix
  ];

  options.services.gamecube-emulator = {
    enable = lib.mkEnableOption "GameCube kiosk with Dolphin emulator";

    user = lib.mkOption {
      type = lib.types.str;
      default = "gamecube";
      description = "User account for the emulator";
    };

    ssh = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Enable SSH for remote management";
      };

      authorizedKeys = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [ ];
        example = [ "ssh-ed25519 AAAA..." ];
        description = "SSH public keys for root access";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    # User
    users.users.${cfg.user} = {
      isNormalUser = true;
      description = "GameCube User";
      extraGroups = [
        "input"
        "video"
        "audio"
        "seat"
      ]
      ++ lib.optional cfg.enableBluetooth "bluetooth";
      home = "/home/${cfg.user}";
      createHome = true;
    };

    # Root authorized keys
    users.users.root.openssh.authorizedKeys.keys = cfg.ssh.authorizedKeys;

    # SSH
    services.openssh = lib.mkIf cfg.ssh.enable {
      enable = true;
      settings = {
        PermitRootLogin = "prohibit-password";
        PasswordAuthentication = false;
      };
    };
  };
}
