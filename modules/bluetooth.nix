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
          Experimental = true;
          # Auto-trust devices for easier pairing
          JustWorksRepairing = "always";
          # Allow faster reconnection
          FastConnectable = true;
          # Keep adapter in bondable mode so link keys are persisted.
          # Without this, bluez sets Bondable=Disabled right before each
          # Pair Device mgmt command, and no [LinkKey] is written to
          # /var/lib/bluetooth/<adapter>/<dev>/info, breaking reboot
          # persistence for HID devices like the Switch Pro Controller.
          AlwaysPairable = true;
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

    # Disable Bluetooth ERTM so Pro Controllers (and other HID gamepads)
    # actually persist their link key during bonding. Without this, the
    # device pairs as Trusted but no [LinkKey] is written to
    # /var/lib/bluetooth, so the bond is lost on the next reboot.
    boot.extraModprobeConfig = ''
      options bluetooth disable_ertm=Y
    '';

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
