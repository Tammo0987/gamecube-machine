{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.gamecube-emulator;

  # Controller navigation script using evsieve
  # Two modes toggled by HOME button:
  #   - NAV mode (default): D-pad->arrows, A->Enter, B->Escape, controller blocked
  #   - GAME mode: Controller passes through, HOME->Escape+toggle back to NAV
  controllerNavScript = pkgs.writeShellScript "controller-nav" ''
    # Wait for the controller device to be ready
    sleep 1

    # Find the Pro Controller event device (works for both USB and Bluetooth)
    # Look for "Pro Controller" (not IMU) in /proc/bus/input/devices
    EVENT_NUM=$(grep -A 10 'Name="Pro Controller"' /proc/bus/input/devices | grep -v IMU | grep 'Handlers=.*event' | grep -oP 'event\d+' | head -1)

    if [ -n "$EVENT_NUM" ]; then
      CONTROLLER="/dev/input/$EVENT_NUM"
    fi

    if [ -z "$CONTROLLER" ]; then
      echo "No Pro Controller found, exiting"
      exit 1
    fi

    echo "Starting controller navigation mapping for: $CONTROLLER"
    echo "Press HOME button to toggle between menu navigation and game mode"
    echo "Starting in NAVIGATION mode (D-pad=arrows, A=Enter, B=Escape)"

    exec ${pkgs.evsieve}/bin/evsieve \
      --input "$CONTROLLER" grab \
      \
      `# Toggle between navigation mode and game mode using HOME button` \
      --hook btn:mode toggle \
      --toggle "" @nav @game \
      \
      `# Navigation mode: D-pad becomes arrow keys` \
      --copy abs:hat0x:-1@nav      key:left:1@kb \
      --copy abs:hat0x:-1..0~@nav  key:left:0@kb \
      --copy abs:hat0x:1@nav       key:right:1@kb \
      --copy abs:hat0x:1..~0@nav   key:right:0@kb \
      --copy abs:hat0y:-1@nav      key:up:1@kb \
      --copy abs:hat0y:-1..0~@nav  key:up:0@kb \
      --copy abs:hat0y:1@nav       key:down:1@kb \
      --copy abs:hat0y:1..~0@nav   key:down:0@kb \
      \
      `# Navigation mode: A button becomes Enter, B button becomes Escape` \
      --copy btn:east@nav key:enter@kb \
      --copy btn:south@nav key:esc@kb \
      \
      `# Block all controller events in navigation mode (only keyboard events pass through)` \
      --block @nav \
      \
      `# Game mode: pass through all controller events to virtual gamepad` \
      --output @game name="Nintendo Switch Pro Controller" \
      \
      `# Output keyboard events (from navigation mode)` \
      --output @kb repeat
  '';

in
{
  options.services.gamecube-emulator = {
    enableControllerNavigation = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Enable controller-to-keyboard mapping for Dolphin menu navigation";
    };
  };

  config = lib.mkIf (cfg.enable && cfg.enableControllerNavigation) {
    # Packages
    environment.systemPackages = [ pkgs.evsieve ];

    # Udev rules for controller navigation
    services.udev.extraRules = ''
      # Nintendo Pro Controller connected - restart navigation mapping
      ACTION=="add", ATTRS{idVendor}=="057e", ATTRS{idProduct}=="2009", TAG+="systemd", ENV{SYSTEMD_WANTS}="controller-navigation.service"
    '';

    # Controller navigation service (evsieve)
    # Allows using Pro Controller D-pad/buttons to navigate Dolphin menus
    # Press HOME button to toggle between NAV mode and GAME mode
    systemd.services.controller-navigation = {
      description = "Controller to keyboard mapping for Dolphin menu navigation";
      wantedBy = [ "multi-user.target" ];
      after = [ "systemd-udevd.service" ];

      serviceConfig = {
        Type = "simple";
        ExecStart = controllerNavScript;
        Restart = "on-failure";
        RestartSec = "5s";
      };
    };
  };
}
