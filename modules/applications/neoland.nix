{ config, lib, pkgs, ... }:

let
  cfg = config.services.neoland;
  neolandPackage = import ../../nix/package.nix {
    inherit pkgs;
    src = ../..;
  };

  execArgs = [
    "server"
    "--grpc-port"
    (toString cfg.grpcPort)
    "--rest-port"
    (toString cfg.restPort)
  ] ++ cfg.extraArgs;

  serviceEnvironment = {
    AUDIT_LOG_PATH = cfg.auditLogPath;
    LOG_FORMAT = cfg.logFormat;
    RUST_LOG = cfg.rustLog;
    NEOLAND_GRPC_PORT = toString cfg.grpcPort;
    NEOLAND_REST_PORT = toString cfg.restPort;
    NEOLAND_DSPY_URL = cfg.dspyUrl;
    NEOLAND_CHECKPOINT_DIR = cfg.checkpointDir;
    NEOLAND_SHM_PATH = cfg.shmPath;
    HOME = "/var/lib/neoland";
    XDG_CACHE_HOME = "/var/cache/neoland";
    HF_HOME = "/var/cache/neoland/huggingface";
  } // cfg.extraEnvironment;
in
{
  options.services.neoland = {
    enable = lib.mkEnableOption "the Neoland control-plane service";

    package = lib.mkOption {
      type = lib.types.package;
      default = neolandPackage;
      defaultText = lib.literalExpression "import ../../nix/package.nix { inherit pkgs; src = ../..; }";
      description = "The Neoland package to run.";
    };

    grpcPort = lib.mkOption {
      type = lib.types.port;
      default = 50051;
      description = "Port for the gRPC control-plane endpoint.";
    };

    restPort = lib.mkOption {
      type = lib.types.port;
      default = 3001;
      description = "Port for the REST control-plane endpoint.";
    };

    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Open the configured gRPC and REST ports in the firewall.";
    };

    dspyUrl = lib.mkOption {
      type = lib.types.str;
      default = "http://127.0.0.1:8001";
      description = "Base URL for the DSPy pipeline service.";
    };

    checkpointDir = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/neoland/checkpoints/adr";
      description = "Filesystem path where ADR checkpoints are written.";
    };

    shmPath = lib.mkOption {
      type = lib.types.str;
      default = "/run/neoland/agent-flags.shm";
      description = "Shared-memory path for mmap IPC flags.";
    };

    auditLogPath = lib.mkOption {
      type = lib.types.str;
      default = "/var/log/neoland/audit.log";
      description = "Audit log file used by the Rust control plane.";
    };

    logFormat = lib.mkOption {
      type = lib.types.enum [ "pretty" "json" "compact" ];
      default = "json";
      description = "Structured log output format for the service.";
    };

    rustLog = lib.mkOption {
      type = lib.types.str;
      default = "info";
      description = "Value for the RUST_LOG environment variable.";
    };

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "/run/secrets/neoland.env";
      description = ''
        Optional systemd EnvironmentFile for secrets and deployment-specific
        variables such as DATABASE_URL, NEOLAND_*_API_KEY, VAULT_ADDR, or
        VAULT_TOKEN.
      '';
    };

    extraEnvironment = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      description = "Additional non-secret environment variables for the service.";
    };

    extraArgs = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      description = "Extra CLI arguments appended to `neoland server`.";
    };
  };

  config = lib.mkIf cfg.enable {
    networking.firewall.allowedTCPPorts =
      lib.optionals cfg.openFirewall [ cfg.grpcPort cfg.restPort ];

    systemd.services.neoland = {
      description = "Neoland control plane";
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];
      environment = serviceEnvironment;
      path = with pkgs; [ coreutils ];
      preStart = ''
        mkdir -p ${lib.escapeShellArg cfg.checkpointDir}
      '';
      serviceConfig = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/neoland ${lib.escapeShellArgs execArgs}";
        WorkingDirectory = "/var/lib/neoland";
        DynamicUser = true;
        Restart = "on-failure";
        RestartSec = "5s";
        CacheDirectory = [ "neoland" "neoland/huggingface" ];
        StateDirectory = [ "neoland" "neoland/checkpoints" "neoland/checkpoints/adr" ];
        LogsDirectory = "neoland";
        RuntimeDirectory = "neoland";
        NoNewPrivileges = true;
        PrivateTmp = true;
        UMask = "0077";
      } // lib.optionalAttrs (cfg.environmentFile != null) {
        EnvironmentFile = cfg.environmentFile;
      };
    };
  };
}
