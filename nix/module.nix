{
  lib,
  pkgs,
  config,
  ...
}:
let
  inherit (lib)
    getExe
    mkIf
    mkMerge
    mkOption
    mkEnableOption
    ;

  cfg = config.services.nixpkgs-prs-bot;

  common = {
    Type = "oneshot";
    User = "nixpkgs-prs-bot";
    Group = "nixpkgs-prs-bot";
    ReadWritePaths = [ cfg.home ];
    LockPersonality = true;
    MemoryDenyWriteExecute = true;
    NoNewPrivileges = true;
    PrivateDevices = true;
    PrivateIPC = true;
    PrivateTmp = true;
    PrivateUsers = true;
    ProtectClock = true;
    ProtectControlGroups = true;
    ProtectHome = true;
    ProtectHostname = true;
    ProtectKernelLogs = true;
    ProtectKernelModules = true;
    ProtectKernelTunables = true;
    ProtectProc = "invisible";
    ProtectSystem = "strict";
    RestrictNamespaces = "uts ipc pid user cgroup";
    RestrictRealtime = true;
    RestrictSUIDSGID = true;
    SystemCallArchitectures = "native";
    SystemCallFilter = [ "@system-service" ];
    UMask = "0077";
  };
in
{
  _class = "nixos";

  options.services.nixpkgs-prs-bot = {
    enable = mkEnableOption "nixpkgs prs bot";

    package = mkOption {
      type = lib.types.package;
      default = pkgs.callPackage ./package.nix { };
    };

    home = mkOption {
      type = lib.types.path;
      default = "/var/lib/nixpkgs-prs-bot";
    };

    fedi = {
      enable = mkEnableOption "fedi" // {
        default = cfg.enable;
      };

      environmentFile = mkOption {
        type = lib.types.nullOr lib.types.path;
        default = null;
      };
    };

    bsky = {
      enable = mkEnableOption "bsky" // {
        default = cfg.enable;
      };

      environmentFile = mkOption {
        type = lib.types.nullOr lib.types.path;
        default = null;
      };
    };
  };

  config = mkIf cfg.enable {
    users = {
      users.nixpkgs-prs-bot = {
        isSystemUser = true;
        inherit (cfg) home;
        createHome = true;
        description = "nixpkgs prs bot";
        group = "nixpkgs-prs-bot";
      };

      groups.nixpkgs-prs-bot = { };
    };

    systemd = {
      timers.nixpkgs-prs = {
        description = "post to fedi/bsky every night";
        wantedBy = [ "timers.target" ];
        timerConfig = {
          OnCalendar = "*-*-* 00:05:00 UTC";
          Persistent = true;
        };
      };

      services = mkMerge [
        (mkIf cfg.fedi.enable {
          nixpkgs-prs-fedibot = {
            description = "nixpkgs prs fedi bot";
            after = [ "network.target" ];
            path = [ cfg.package ];

            serviceConfig = {
              ExecStart = "${getExe cfg.package} fedi";
              EnvironmentFile = mkIf (cfg.fedi.environmentFile != null) cfg.bsky.environmentFile;
            } // common;
          };
        })

        (mkIf cfg.bsky.enable {
          nixpkgs-prs-bskybot = {
            description = "nixpkgs prs bsky bot";
            after = [ "network.target" ];
            path = [ cfg.package ];

            serviceConfig = {
              ExecStart = "${getExe cfg.package} bsky";
              EnvironmentFile = mkIf (cfg.bsky.environmentFile != null) cfg.bsky.environmentFile;
            } // common;
          };
        })
      ];
    };
  };
}
