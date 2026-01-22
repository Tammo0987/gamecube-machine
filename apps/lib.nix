{ pkgs }:

{
  # Common shell script header with colors and utilities
  scriptHeader = ''
    set -euo pipefail

    # Colors
    GREEN='\033[0;32m'
    RED='\033[0;31m'
    BLUE='\033[0;34m'
    YELLOW='\033[1;33m'
    NC='\033[0m'

    # Logging functions
    info() { echo -e "''${GREEN}[INFO]''${NC} $1"; }
    error() { echo -e "''${RED}[ERROR]''${NC} $1"; exit 1; }
    step() { echo -e "''${BLUE}[STEP]''${NC} $1"; }
    warn() { echo -e "''${YELLOW}[WARN]''${NC} $1"; }
  '';

  # SSH connection check function
  mkSshCheck =
    { host, user }:
    ''
      check_ssh() {
        step "Checking SSH connection to ${host}..."
        if ! ${pkgs.openssh}/bin/ssh -o ConnectTimeout=5 "${user}@${host}" "echo ok" &>/dev/null; then
          error "Cannot connect to ${host}"
        fi
        info "SSH connection successful!"
      }
    '';

  # SSH command helper
  ssh = pkgs.openssh;
}
