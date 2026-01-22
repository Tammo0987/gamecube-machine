# GameCube Machine - NixOS Kiosk Module

> **Note:** This project is currently for personal use only. You're welcome to try it, but don't expect it to work out of the box for your setup. No support is provided.

A NixOS module that transforms any NixOS system into a dedicated GameCube/Wii gaming kiosk using Dolphin emulator.

## Features

- Auto-boots directly into Dolphin emulator (fullscreen)
- Sway Wayland compositor in kiosk mode
- Nintendo Switch Pro Controller support via Bluetooth
- Controller-based menu navigation (D-pad, A/B buttons)
- Configurable graphics settings (resolution, MSAA, VSync)
- SSH server for remote management

## Usage

### Add to Your Flake

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    gamecube-machine.url = "github:Tammo0987/gamecube-machine";
  };

  outputs = { nixpkgs, gamecube-machine, ... }:
    gamecube-machine.lib.mkSystem {
      host = "192.168.1.100";
      gamesLocal = "/path/to/local/games";
      modules = [
        ./hardware-configuration.nix
        # your other modules
      ];
    };
}
```

### Deploy and Sync

```bash
nix run .#deploy          # Deploy configuration
nix run .#sync-games      # Sync game ISOs
nix run .#pair-controller # Pair Bluetooth controller
```

## Configuration Options

### General

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enable` | bool | `false` | Enable the kiosk mode |
| `user` | string | `"gamecube"` | User account for kiosk |
| `gamesDirectory` | path | `"/var/lib/dolphin-emu/games"` | ISO storage location |
| `enableBluetooth` | bool | `true` | Enable Bluetooth support |
| `enableControllerNavigation` | bool | `true` | Enable controller menu navigation |
| `theme` | string | `"Clean Emerald"` | Dolphin UI theme |

### SSH

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `ssh.enable` | bool | `true` | Enable SSH server |
| `ssh.authorizedKeys` | list | `[]` | SSH public keys for root access (required) |

> **Note:** Password authentication is disabled. You must provide SSH public keys via `ssh.authorizedKeys` for remote access.

### Graphics

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `graphics.internalResolution` | int | `2` | Resolution multiplier (1=native, 2=2x, 4=4x) |
| `graphics.msaa` | enum | `2` | Anti-aliasing (0, 2, 4, 8) |
| `graphics.vsync` | bool | `true` | Enable vertical sync |
| `graphics.showFPS` | bool | `false` | Show FPS counter |
| `graphics.aspectRatio` | enum | `0` | Aspect ratio (0=auto, 1=16:9, 2=4:3, 3=stretch) |
| `graphics.waitForShaders` | bool | `true` | Compile shaders before game start |

## Controller Navigation

When `enableControllerNavigation` is enabled, the Nintendo Switch Pro Controller can navigate Dolphin menus:

- **NAV mode** (default): D-pad = arrows, A = Enter, B = Escape
- **GAME mode**: Controller passes through to emulator
- **HOME button**: Toggle between NAV and GAME mode
