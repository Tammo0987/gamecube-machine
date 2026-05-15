{
  pkgs,
  lib,
  config,
}:

pkgs.writeShellScript "sync-games" ''
  ${lib.scriptHeader}
  ${lib.mkSshCheck { inherit (config) host user; }}

  HOST="${config.host}"
  USER="${config.user}"
  LOCAL_DIR="${config.gamesLocal}"
  REMOTE_DIR="${config.gamesRemote}"

  if [ ! -d "$LOCAL_DIR" ]; then
    error "Local games directory not found: $LOCAL_DIR"
  fi

  check_ssh

  ${lib.ssh}/bin/ssh "$USER@$HOST" "mkdir -p $REMOTE_DIR"

  step "Syncing games and cover art..."
  ${pkgs.rsync}/bin/rsync -azh --progress \
    --include="*.iso" --include="*.ISO" \
    --include="*.gcm" --include="*.GCM" \
    --include="*.wbfs" --include="*.WBFS" \
    --include="*.rvz" --include="*.RVZ" \
    --include="*.jpg" --include="*.JPG" \
    --include="*.jpeg" --include="*.JPEG" \
    --include="*.png" --include="*.PNG" \
    --exclude="*" \
    "$LOCAL_DIR/" "$USER@$HOST:$REMOTE_DIR/" 2>&1 | \
    grep -v "^$" | grep -v "^sending" | grep -v "^total size" | grep -v "^sent" || true

  ${lib.ssh}/bin/ssh "$USER@$HOST" "chmod -R 755 $REMOTE_DIR" > /dev/null

  info "Games synced!"
  echo ""

  ${lib.ssh}/bin/ssh "$USER@$HOST" "
    for f in \"$REMOTE_DIR\"/*; do
      [ -f \"\$f\" ] || continue
      size=\$(du -h \"\$f\" | cut -f1)
      name=\$(basename \"\$f\")
      name=\"\''${name%.*}\"
      echo \"\$name|\$size\"
    done
  " | while IFS='|' read -r name size; do
    echo -e "  ''${GREEN}●''${NC} $name ''${BLUE}($size)''${NC}"
  done
''
