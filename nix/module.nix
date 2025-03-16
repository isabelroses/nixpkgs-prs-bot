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
      type = lib.types.str;
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

    systemd = mkMerge (
      lib.map
        (
          attr:
          mkIf cfg.${attr}.enable {
            timers."nixpkgs-prs-${attr}" = {
              description = "post to ${attr} every night";
              wantedBy = [ "timers.target" ];
              timerConfig = {
                OnCalendar = "*-*-* 00:05:00 UTC";
                Persistent = true;
              };
            };

            services."nixpkgs-prs-${attr}" = {
              description = "nixpkgs prs ${attr} bot";
              after = [ "network.target" ];
              path = [ cfg.package ];

              serviceConfig = {
                ExecStart = "${getExe cfg.package} ${attr}";
                EnvironmentFile = mkIf (cfg.${attr}.environmentFile != null) cfg.${attr}.environmentFile;
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
            };
          }
        )
        [
          "fedi"
          "bsky"
        ]
    );
  };
}
