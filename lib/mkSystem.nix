{ nixpkgs, module, apps }:

{
  host,
  gamesLocal,
  modules ? [],
  user ? "root",
  configName ? "gamecube",
}:

let
  system = "x86_64-linux";

  nixosConfig = nixpkgs.lib.nixosSystem {
    inherit system;
    modules = modules ++ [
      module
      {
        networking.hostName = configName;
        services.gamecube-emulator.enable = true;
      }
    ];
  };

  cfg = nixosConfig.config.services.gamecube-emulator;
in
{
  nixosConfigurations.${configName} = nixosConfig;

  apps.${system} = {
    deploy = apps.mkDeploy {
      inherit host user;
      toplevel = nixosConfig.config.system.build.toplevel;
    };
    sync-games = apps.mkSyncGames {
      inherit host user gamesLocal;
      gamesRemote = cfg.gamesDirectory;
    };
    pair-controller = apps.mkPairController {
      inherit host user;
    };
  };
}
