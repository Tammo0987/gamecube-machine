{
  pkgs,
  lib,
  config,
}:

pkgs.writeShellScript "pair-controller" ''
    ${lib.scriptHeader}

    HOST="${config.host}"
    USER="${config.user}"

    echo -e "''${BLUE}Nintendo Switch Pro Controller Bluetooth Pairing''${NC}"
    echo "=================================================="
    echo ""
    echo "Instructions:"
    echo "1. Hold the SYNC button on your Pro Controller"
    echo "   (small button on top, near USB-C port)"
    echo "2. The player LEDs will start flashing"
    echo "3. Keep holding until pairing completes"
    echo ""
    read -p "Press Enter when controller is in pairing mode..."

    step "Connecting to $HOST..."
    ${lib.ssh}/bin/ssh "$USER@$HOST" << 'REMOTE_EOF'
      echo "Starting Bluetooth scan..."

      # Run bluetoothctl with scan and capture output
      {
        echo "power on"
        echo "scan on"
        sleep 10
        echo "scan off"
        echo "quit"
      } | bluetoothctl 2>&1 > /tmp/bt_scan.log

      echo "Scanning complete, looking for Pro Controller..."

      # Find the controller MAC address
      CONTROLLER_MAC=$(grep -i "Pro Controller" /tmp/bt_scan.log | grep -ioE '([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}' | head -1)

      if [ -z "$CONTROLLER_MAC" ]; then
        echo "ERROR: No Pro Controller found. Make sure LEDs are flashing!"
        rm -f /tmp/bt_scan.log
        exit 1
      fi

      echo "Found Pro Controller: $CONTROLLER_MAC"
      rm -f /tmp/bt_scan.log

      echo "Pairing..."
      bluetoothctl pair "$CONTROLLER_MAC"
      sleep 2

      echo "Trusting device..."
      bluetoothctl trust "$CONTROLLER_MAC"
      sleep 2

      echo "Connecting..."
      bluetoothctl connect "$CONTROLLER_MAC"
      sleep 2

      echo ""
      echo "Pairing complete!"
      echo ""
      echo "Press L+R buttons together on your Pro Controller to activate with joycond."
  REMOTE_EOF

    if [ $? -eq 0 ]; then
      info "Controller paired successfully!"
      echo ""
      echo "The controller will auto-reconnect when powered on."
    else
      error "Pairing failed. Try again with controller LEDs flashing."
    fi
''
