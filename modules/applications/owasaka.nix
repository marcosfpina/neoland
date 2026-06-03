# O.W.A.S.A.K.A — NixOS Module
#
# Security analysis and policy enforcement service for the Neoland stack.
# Connects to the Spectre NATS bus to observe pipeline events and flag
# policy violations, vulnerable patterns, and security anomalies in real-time.
#
# Usage:
#   services.owasaka = {
#     enable = true;
#     natsUrl = config.services.spectre-event-bus.natsUrl;
#     neolandUrl = "http://127.0.0.1:3001";
#     environmentFile = "/run/secrets/owasaka.env";
#   };

{ inputs }:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.owasaka;
  owasakaPackage = inputs.owasaka.packages.${pkgs.system}.default;

  serviceEnvironment = {
    OWASAKA_HOST          = cfg.host;
    OWASAKA_PORT          = toString cfg.port;
    OWASAKA_NATS_URL      = cfg.natsUrl;
    OWASAKA_NEOLAND_URL   = cfg.neolandUrl;
    OWASAKA_LOG_LEVEL     = cfg.logLevel;
    OWASAKA_POLICY_DIR    = cfg.policyDir;
    RUST_LOG              = cfg.logLevel;
  } // cfg.extraEnvironment;
in
{
  options.services.owasaka = {
    enable = lib.mkEnableOption "O.W.A.S.A.K.A. security analysis service";

    package = lib.mkOption {
      type = lib.types.package;
      default = owasakaPackage;
      defaultText = lib.literalExpression "inputs.owasaka.packages.\${pkgs.system}.default";
      description = "The OWASAKA package.";
    };

    host = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Bind address for the OWASAKA API.";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 7777;
      description = "HTTP port for the OWASAKA API and webhook receiver.";
    };

    natsUrl = lib.mkOption {
      type = lib.types.str;
      default = "nats://127.0.0.1:4222";
      description = "NATS URL of the Spectre event bus to subscribe to.";
    };

    neolandUrl = lib.mkOption {
      type = lib.types.str;
      default = "http://127.0.0.1:3001";
      description = "Neoland control-plane REST URL for policy alerts.";
    };

    policyDir = lib.mkOption {
      type = lib.types.str;
      default = "/etc/owasaka/policies";
      description = "Directory of OWASAKA policy definition files (TOML/YAML).";
    };

    logLevel = lib.mkOption {
      type = lib.types.str;
      default = "info";
      description = "Log level (RUST_LOG value).";
    };

    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Open the OWASAKA port in the firewall.";
    };

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "/run/secrets/owasaka.env";
      description = "systemd EnvironmentFile for secrets (OWASAKA_API_KEY, etc.).";
    };

    extraEnvironment = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      description = "Additional environment variables.";
    };
  };

  config = lib.mkIf cfg.enable {
    networking.firewall.allowedTCPPorts =
      lib.optionals cfg.openFirewall [ cfg.port ];

    systemd.services.owasaka = {
      description = "O.W.A.S.A.K.A. security analysis and policy enforcement";
      after = [ "network-online.target" "spectre-nats.service" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];
      environment = serviceEnvironment;
      serviceConfig = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/owasaka";
        DynamicUser = true;
        StateDirectory = "owasaka";
        LogsDirectory = "owasaka";
        ConfigurationDirectory = "owasaka";
        Restart = "on-failure";
        RestartSec = "5s";
        NoNewPrivileges = true;
        PrivateTmp = true;
        UMask = "0077";
      } // lib.optionalAttrs (cfg.environmentFile != null) {
        EnvironmentFile = cfg.environmentFile;
      };
    };
  };
}
