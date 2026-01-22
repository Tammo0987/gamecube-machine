{ pkgs }:

let
  lib = import ./lib.nix { inherit pkgs; };
in
{
  mkDeploy =
    {
      host,
      user,
      toplevel,
    }:
    {
      type = "app";
      program = toString (
        import ./deploy.nix {
          inherit pkgs lib;
          config = { inherit host user toplevel; };
        }
      );
    };

  mkSyncGames =
    {
      host,
      user,
      gamesLocal,
      gamesRemote,
    }:
    {
      type = "app";
      program = toString (
        import ./sync-games.nix {
          inherit pkgs lib;
          config = {
            inherit
              host
              user
              gamesLocal
              gamesRemote
              ;
          };
        }
      );
    };

  mkPairController =
    { host, user }:
    {
      type = "app";
      program = toString (
        import ./pair-controller.nix {
          inherit pkgs lib;
          config = { inherit host user; };
        }
      );
    };
}
