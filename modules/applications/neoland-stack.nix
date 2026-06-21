# Neoland Full Stack — Composite NixOS Module
#
# Wires together Neoland + Spectre Event Bus + OWASAKA into a single,
# opinionated deployment. All inter-service URLs are auto-configured;
# individual service options remain tunable via their own namespaces.
#
# Minimal usage (all defaults):
#   imports = [ neolandStackModule ];
#   services.neoland-stack.enable = true;
#   services.neoland-stack.environmentFile = "/run/secrets/neoland-stack.env";
#
# That single option brings up:
#   - Neoland control-plane        :3001 (REST) + :50051 (gRPC)
#   - Spectre NATS event bus       :4222 (NATS) + :8222 (monitor)
#   - Spectre proxy gateway        :4280
#   - O.W.A.S.A.K.A. security      :7777
#
# Advanced usage (override individual services):
#   services.neoland-stack.enable = true;
#   services.spectre-event-bus.environment = "prod";
#   services.neoland.restPort = 8080;
#   services.owasaka.policyDir = "/opt/policies";

{ inputs }:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.neoland-stack;
in
{
  options.services.neoland-stack = {
    enable = lib.mkEnableOption ''
      Neoland full stack — Neoland control-plane + Spectre event bus + OWASAKA security
    '';

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "/run/secrets/neoland-stack.env";
      description = ''
        Shared EnvironmentFile passed to all services in the stack.
        A single SOPS-decrypted file can carry all secrets:
        NEOLAND_ADMIN_API_KEY, NEOLAND_USER_API_KEY, OWASAKA_API_KEY, etc.
      '';
    };

    spectre.enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Enable the Spectre event bus (NATS + proxy). Set false to use an external bus.";
    };

    owasaka.enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Enable the OWASAKA security analysis service.";
    };
  };

  config = lib.mkIf cfg.enable {
    # ── Neoland control-plane ─────────────────────────────────────────
    services.neoland = {
      enable = true;
      environmentFile = cfg.environmentFile;
      # Point Neoland at the Spectre NATS bus for event publishing
      extraEnvironment = lib.mkIf cfg.spectre.enable {
        NEOLAND_NATS_URL = config.services.spectre-event-bus.natsUrl;
      };
    };

    # ── Spectre Event Bus ─────────────────────────────────────────────
    services.spectre-event-bus = lib.mkIf cfg.spectre.enable {
      enable = true;
      environmentFile = cfg.environmentFile;
    };

    # ── O.W.A.S.A.K.A. ───────────────────────────────────────────────
    services.owasaka = lib.mkIf cfg.owasaka.enable {
      enable = true;
      natsUrl = lib.mkIf cfg.spectre.enable
        config.services.spectre-event-bus.natsUrl;
      neolandUrl =
        "http://127.0.0.1:${toString config.services.neoland.restPort}";
      environmentFile = cfg.environmentFile;
    };
  };
}
