{ inputs }:
{ config, lib, pkgs, ... }:

let
  cfg = config.services.securellm-bridge-api;
  securellmPackage = inputs.securellmBridge.packages.${pkgs.system}.default;

  serviceEnvironment = {
    SERVER_HOST = cfg.host;
    SERVER_PORT = toString cfg.port;
    DATABASE_URL = cfg.databaseUrl;
    REDIS_URL = cfg.redisUrl;
    LOG_DIR = cfg.logDir;
    REQUIRE_AUTH = lib.boolToString cfg.requireAuth;
    LOG_LEVEL = cfg.logLevel;
    ML_OPS_ENABLED = lib.boolToString cfg.mlOps.enable;
    ML_OPS_API_URL = cfg.mlOps.apiUrl;
    LLAMACPP_ENABLED = lib.boolToString cfg.llamacpp.enable;
    LLAMACPP_BASE_URL = cfg.llamacpp.baseUrl;
    HOME = cfg.dataDir;
  } // cfg.extraEnvironment;
in
{
  options.services.securellm-bridge-api = {
    enable = lib.mkEnableOption "SecureLLM Bridge API gateway";

    package = lib.mkOption {
      type = lib.types.package;
      default = securellmPackage;
      defaultText = lib.literalExpression "inputs.securellmBridge.packages.${pkgs.system}.default";
      description = "The SecureLLM Bridge package providing `securellm-api-server`.";
    };

    host = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Bind address for the SecureLLM API gateway.";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 8080;
      description = "Bind port for the SecureLLM API gateway.";
    };

    dataDir = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/securellm";
      description = "State directory for SQLite data and runtime files.";
    };

    logDir = lib.mkOption {
      type = lib.types.str;
      default = "/var/log/securellm";
      description = "Directory where SecureLLM writes structured logs.";
    };

    databaseUrl = lib.mkOption {
      type = lib.types.str;
      default = "sqlite:/var/lib/securellm/models.db";
      description = "SQLite database URL used by the SecureLLM API server.";
    };

    redisUrl = lib.mkOption {
      type = lib.types.str;
      default = "redis://localhost:6379";
      description = "Redis URL used by the SecureLLM API server.";
    };

    requireAuth = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Require API key authentication at the SecureLLM gateway.";
    };

    logLevel = lib.mkOption {
      type = lib.types.str;
      default = "info";
      description = "Value for the SecureLLM `LOG_LEVEL` environment variable.";
    };

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        Optional systemd EnvironmentFile for provider keys and deployment
        variables such as `DEEPSEEK_API_KEY`, `OPENAI_API_KEY`, or `API_KEYS`.
      '';
    };

    extraEnvironment = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      description = "Additional non-secret environment variables for the gateway.";
    };

    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Open the SecureLLM gateway port in the firewall.";
    };

    mlOps = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Enable `ml-ops-api` as an upstream provider.";
      };

      apiUrl = lib.mkOption {
        type = lib.types.str;
        default = "http://127.0.0.1:8083";
        description = "Base URL for the upstream `ml-ops-api` service.";
      };
    };

    llamacpp = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Expose direct llama.cpp as a provider at the gateway level.";
      };

      baseUrl = lib.mkOption {
        type = lib.types.str;
        default = "http://127.0.0.1:5001";
        description = "Base URL for a direct llama.cpp upstream, if enabled.";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    networking.firewall.allowedTCPPorts = lib.optionals cfg.openFirewall [ cfg.port ];

    users.users.securellm = {
      isSystemUser = true;
      group = "securellm";
      home = cfg.dataDir;
      createHome = false;
    };
    users.groups.securellm = { };

    systemd.tmpfiles.rules = [
      "d ${cfg.dataDir} 0750 securellm securellm -"
      "d ${cfg.logDir} 0750 securellm securellm -"
    ];

    systemd.services.securellm-bridge-api = {
      description = "SecureLLM Bridge API gateway";
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ] ++ lib.optionals cfg.mlOps.enable [ "ml-ops-api.service" ];
      wants = [ "network-online.target" ] ++ lib.optionals cfg.mlOps.enable [ "ml-ops-api.service" ];
      environment = serviceEnvironment;
      path = with pkgs; [ coreutils ];
      serviceConfig =
        {
          Type = "simple";
          ExecStart = "${cfg.package}/bin/securellm-api-server";
          User = "securellm";
          Group = "securellm";
          WorkingDirectory = cfg.dataDir;
          Restart = "on-failure";
          RestartSec = "5s";
          StateDirectory = "securellm";
          LogsDirectory = "securellm";
          NoNewPrivileges = true;
          PrivateTmp = true;
          ProtectSystem = "strict";
          ProtectHome = true;
          ReadWritePaths = [ cfg.dataDir cfg.logDir ];
          UMask = "0077";
        }
        // lib.optionalAttrs (cfg.environmentFile != null) {
          EnvironmentFile = cfg.environmentFile;
        };
    };
  };
}
