{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.gamecube-emulator;
  launcher = pkgs.callPackage ../launcher { };
in
{
  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ launcher ];
  };
}
