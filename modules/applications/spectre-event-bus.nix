# Spectre Event Bus — NixOS module
#
# Wires the Spectre NATS-backed event bus into the Neoland stack.
# Runs two systemd services:
#   - spectre-nats       : JetStream NATS server (event backbone)
#   - spectre-proxy      : Spectre proxy gateway connecting to NATS
#
# The Neoland control plane reads services.spectre-event-bus.natsUrl to
# know where to publish/subscribe agent pipeline events.
#
# Usage:
#   services.spectre-event-bus = {
#     enable = true;
#     environmentFile = "/run/secrets/spectre.env";
#   };
#
# Then point Neoland at it:
#   services.neoland.extraEnvironment.NEOLAND_NATS_URL =
#     config.services.spectre-event-bus.natsUrl;

{ inputs }:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.spectre-event-bus;
  spectrePackage = inputs.spectre.packages.${pkgs.system}.default;

  natsConf = pkgs.writeText "spectre-nats.conf" ''
    server_name = "${cfg.serverName}"
    port = ${toString cfg.natsPort}
    monitor_port = ${toString cfg.natsMonitorPort}

    jetstream {
      store_dir = "${cfg.dataDir}/nats"
      max_memory_store = ${cfg.jetstream.maxMemory}
      max_file_store   = ${cfg.jetstream.maxFile}
    }

    max_connections = ${toString cfg.maxConnections}
    ${lib.optionalString (cfg.authorization != null) cfg.authorization}
  '';

  proxyEnvironment = {
    NATS_URL       = cfg.natsUrl;
    SPECTRE_ENV    = cfg.environment;
    RUST_LOG       = cfg.logLevel;
  } // cfg.extraEnvironment;
in
{
  options.services.spectre-event-bus = {
    enable = lib.mkEnableOption "Spectre event bus (NATS + spectre-proxy)";

    package = lib.mkOption {
      type = lib.types.package;
      default = spectrePackage;
      defaultText = lib.literalExpression "inputs.spectre.packages.\${pkgs.system}.default";
      description = "spectre-proxy package.";
    };

    serverName = lib.mkOption {
      type = lib.types.str;
      default = "neoland-spectre-nats";
      description = "NATS server name shown in logs and cluster membership.";
    };

    natsPort = lib.mkOption {
      type = lib.types.port;
      default = 4222;
      description = "NATS client connection port.";
    };

    natsMonitorPort = lib.mkOption {
      type = lib.types.port;
      default = 8222;
      description = "NATS HTTP monitoring port.";
    };

    proxyPort = lib.mkOption {
      type = lib.types.port;
      default = 4280;
      description = "Spectre proxy HTTP/gRPC gateway port.";
    };

    # Convenience: computed from natsPort, readable by other modules
    natsUrl = lib.mkOption {
      type = lib.types.str;
      default = "nats://127.0.0.1:${toString cfg.natsPort}";
      defaultText = lib.literalExpression ''"nats://127.0.0.1:''${toString cfg.natsPort}"'';
      description = "NATS URL for other services (Neoland, ai-reactor) to connect.";
    };

    environment = lib.mkOption {
      type = lib.types.enum [ "dev" "staging" "prod" ];
      default = "dev";
      description = "Deployment environment tag propagated as SPECTRE_ENV.";
    };

    dataDir = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/spectre";
      description = "State directory for JetStream persistence.";
    };

    jetstream = {
      maxMemory = lib.mkOption {
        type = lib.types.str;
        default = "256MB";
        description = "JetStream max in-memory store size.";
      };
      maxFile = lib.mkOption {
        type = lib.types.str;
        default = "1GB";
        description = "JetStream max file store size.";
      };
    };

    maxConnections = lib.mkOption {
      type = lib.types.int;
      default = 512;
      description = "Maximum NATS client connections.";
    };

    authorization = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = ''
        authorization {
          token = "$NATS_AUTH_TOKEN"
        }
      '';
      description = "Raw NATS authorization config block, or null for no auth.";
    };

    logLevel = lib.mkOption {
      type = lib.types.str;
      default = "info";
      description = "RUST_LOG level for spectre-proxy.";
    };

    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Open NATS and proxy ports in the firewall.";
    };

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "/run/secrets/spectre.env";
      description = "systemd EnvironmentFile for secrets (NATS_AUTH_TOKEN, etc.).";
    };

    extraEnvironment = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      description = "Additional environment variables for spectre-proxy.";
    };
  };

  config = lib.mkIf cfg.enable {
    networking.firewall.allowedTCPPorts = lib.optionals cfg.openFirewall [
      cfg.natsPort
      cfg.natsMonitorPort
      cfg.proxyPort
    ];

    # ── NATS JetStream server ─────────────────────────────────────────
    systemd.services.spectre-nats = {
      description = "Spectre NATS JetStream event backbone";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      preStart = "mkdir -p ${cfg.dataDir}/nats";
      serviceConfig = {
        Type = "simple";
        ExecStart = "${pkgs.nats-server}/bin/nats-server -c ${natsConf}";
        WorkingDirectory = cfg.dataDir;
        DynamicUser = true;
        StateDirectory = "spectre/nats";
        Restart = "on-failure";
        RestartSec = "3s";
        NoNewPrivileges = true;
        PrivateTmp = true;
      };
    };

    # ── Spectre proxy gateway ─────────────────────────────────────────
    systemd.services.spectre-proxy = {
      description = "Spectre event proxy gateway";
      after = [ "network.target" "spectre-nats.service" ];
      requires = [ "spectre-nats.service" ];
      wantedBy = [ "multi-user.target" ];
      environment = proxyEnvironment;
      serviceConfig = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/spectre-proxy";
        DynamicUser = true;
        StateDirectory = "spectre";
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
