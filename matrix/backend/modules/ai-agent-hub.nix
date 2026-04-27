{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.ai-agent-hub;
in
{
  options.services.ai-agent-hub = {
    enable = mkEnableOption "AI Agent Hub Observability Service";
    
    port = mkOption {
      type = types.port;
      default = 8090;
      description = "Port for the Ranking API";
    };

    observability = {
      prometheus = mkEnableOption "Enable internal Prometheus instance";
      retention = mkOption {
        type = types.str;
        default = "15d";
        description = "Prometheus data retention";
      };
    };
  };

  config = mkIf cfg.enable {
    # Systemd service for Ranking API
    systemd.services.ai-agent-ranking = {
      description = "AI Agent Ranking API";
      wantedBy = [ "multi-user.target" ];
      environment = {
        PORT = toString cfg.port;
      };
      serviceConfig = {
        ExecStart = "${pkgs.python3}/bin/python -m uvicorn src.ranking.main:app --host 0.0.0.0 --port ${toString cfg.port}";
        Restart = "always";
        User = "ai-hub";
        DynamicUser = true;
      };
    };

    # Optional internal Prometheus
    services.prometheus = mkIf cfg.observability.prometheus {
      enable = true;
      retentionTime = cfg.observability.retention;
      scrapeConfigs = [
        {
          job_name = "ai-agents";
          static_configs = [{
            targets = [ "localhost:${toString cfg.port}" ];
          }];
        }
      ];
    };
  };
}
