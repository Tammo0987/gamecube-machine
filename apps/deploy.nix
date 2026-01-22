{
  pkgs,
  lib,
  config,
}:

pkgs.writeShellScript "deploy" ''
  ${lib.scriptHeader}
  ${lib.mkSshCheck { inherit (config) host user; }}

  HOST="${config.host}"
  USER="${config.user}"
  STORE_PATH="${config.toplevel}"

  check_ssh

  info "Store path: $STORE_PATH"

  step "Copying closure to $HOST..."
  ${pkgs.nix}/bin/nix copy --to "ssh://$USER@$HOST" "$STORE_PATH"

  step "Activating configuration..."
  ${lib.ssh}/bin/ssh "$USER@$HOST" "nix-env -p /nix/var/nix/profiles/system --set $STORE_PATH && $STORE_PATH/bin/switch-to-configuration switch"

  info "Deployment complete!"

  echo ""
  read -p "Reboot now? [y/N]: " -n 1 -r
  echo
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    ${lib.ssh}/bin/ssh "$USER@$HOST" "reboot" || true
    info "Reboot initiated"
  fi
''
