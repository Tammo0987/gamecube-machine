{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.gamecube-emulator;

in
{
  options.services.gamecube-emulator = {
    enableBluetooth = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Enable Bluetooth for wireless controllers";
    };
  };

  config = lib.mkIf (cfg.enable && cfg.enableBluetooth) {
    # Packages
    environment.systemPackages = with pkgs; [
      bluez
      joycond
    ];

    # Bluetooth configuration
    hardware.bluetooth = {
      enable = true;
      powerOnBoot = true;
      settings = {
        General = {
          Enable = "Source,Sink,Media,Socket";
          Experimental = true;
          # Auto-trust devices for easier pairing
          JustWorksRepairing = "always";
          # Allow faster reconnection
          FastConnectable = true;
        };
        Policy = {
          # Auto-enable controllers
          AutoEnable = true;
        };
      };
      # Workaround for "Rejected connection from !bonded device" error
      # See: https://github.com/bluez/bluez/issues/824
      # WARNING: This makes the system vulnerable to CVE-2023-45866
      # Only enable Bluetooth discovery when pairing controllers
      input = {
        General = {
          ClassicBondedOnly = false;
          UserspaceHID = true;
        };
      };
    };

    services.blueman.enable = true;

    # Nintendo Switch Pro Controller support
    boot.kernelModules = [ "hid-nintendo" ];

    # Udev rules for Nintendo Switch Pro Controller
    services.udev.extraRules = ''
      # Nintendo Switch Pro Controller (Bluetooth)
      KERNEL=="hidraw*", ATTRS{idVendor}=="057e", ATTRS{idProduct}=="2009", MODE="0660", TAG+="uaccess"
      # Nintendo Switch Pro Controller (USB - for initial setup)
      SUBSYSTEM=="usb", ATTRS{idVendor}=="057e", ATTRS{idProduct}=="2009", MODE="0660", TAG+="uaccess"
    '';

    # Joycond service for Nintendo Switch controller management
    systemd.services.joycond = {
      description = "Joycond daemon for Nintendo Switch controllers";
      wantedBy = [ "multi-user.target" ];
      after = [ "bluetooth.service" ];

      serviceConfig = {
        Type = "simple";
        ExecStart = "${pkgs.joycond}/bin/joycond";
        Restart = "on-failure";
        RestartSec = "5s";
      };
    };
  };
}
