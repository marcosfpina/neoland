{ inputs }:
{ config, lib, pkgs, ... }:

let
  cfg = config.services.ml-ops-api;
  mlOpsPackages = inputs.mlOpsApi.packages.${pkgs.system};
  mlOpsPackage = if mlOpsPackages ? api then mlOpsPackages.api else mlOpsPackages.default;

  serviceEnvironment = {
    ML_OFFLOAD_HOST = cfg.host;
    ML_OFFLOAD_PORT = toString cfg.port;
    ML_OFFLOAD_DATA_DIR = cfg.dataDir;
    ML_OFFLOAD_MODELS_PATH = cfg.modelsPath;
    ML_OFFLOAD_DB_PATH = cfg.dbPath;
    ML_OFFLOAD_CORS_ENABLED = lib.boolToString cfg.corsEnabled;
    LLAMACPP_URL = cfg.llamacppUrl;
    VLLM_URL = cfg.vllmUrl;
    RUST_LOG = cfg.rustLog;
    HOME = cfg.dataDir;
  } // cfg.extraEnvironment;
in
{
  options.services.ml-ops-api = {
    enable = lib.mkEnableOption "ML-Ops API inference bridge";

    package = lib.mkOption {
      type = lib.types.package;
      default = mlOpsPackage;
      defaultText = lib.literalExpression "inputs.mlOpsApi.packages.${pkgs.system}.api";
      description = "The ML-Ops API package providing `ml-offload-api`.";
    };

    host = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Bind address for the inference bridge.";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 8083;
      description = "Bind port for the inference bridge.";
    };

    dataDir = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/ml-offload";
      description = "State directory for the ML-Ops bridge.";
    };

    modelsPath = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/ml-models";
      description = "Directory containing GGUF and other model artifacts.";
    };

    dbPath = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/ml-offload/registry.db";
      description = "SQLite registry path for the inference bridge.";
    };

    llamacppUrl = lib.mkOption {
      type = lib.types.str;
      default = "http://127.0.0.1:5001";
      description = "Base URL of the upstream `llama.cpp` server.";
    };

    vllmUrl = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "Base URL of an optional upstream `vLLM` server.";
    };

    rustLog = lib.mkOption {
      type = lib.types.str;
      default = "info";
      description = "Value for the `RUST_LOG` environment variable.";
    };

    corsEnabled = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Enable permissive CORS on the inference bridge.";
    };

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Optional systemd EnvironmentFile for deployment-specific bridge settings.";
    };

    extraEnvironment = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      description = "Additional non-secret environment variables for the bridge.";
    };

    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Open the ML-Ops API port in the firewall.";
    };
  };

  config = lib.mkIf cfg.enable {
    networking.firewall.allowedTCPPorts = lib.optionals cfg.openFirewall [ cfg.port ];

    users.users.ml-offload = {
      isSystemUser = true;
      group = "ml-offload";
      home = cfg.dataDir;
      createHome = false;
    };
    users.groups.ml-offload = { };

    systemd.tmpfiles.rules = [
      "d ${cfg.dataDir} 0750 ml-offload ml-offload -"
      "d ${cfg.modelsPath} 0750 ml-offload ml-offload -"
    ];

    systemd.services.ml-ops-api = {
      description = "ML-Ops API inference bridge";
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      environment = serviceEnvironment;
      path = with pkgs; [ coreutils ];
      serviceConfig =
        {
          Type = "simple";
          ExecStart = "${cfg.package}/bin/ml-offload-api";
          User = "ml-offload";
          Group = "ml-offload";
          WorkingDirectory = cfg.dataDir;
          Restart = "on-failure";
          RestartSec = "5s";
          StateDirectory = "ml-offload";
          LogsDirectory = "ml-offload";
          NoNewPrivileges = true;
          PrivateTmp = true;
          ProtectSystem = "strict";
          ProtectHome = true;
          ReadWritePaths = [ cfg.dataDir cfg.modelsPath ];
          UMask = "0077";
        }
        // lib.optionalAttrs (cfg.environmentFile != null) {
          EnvironmentFile = cfg.environmentFile;
        };
    };
  };
}
