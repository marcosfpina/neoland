# Mission Control NixOS Module
# Include this in your configuration.nix

{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.mission-control;
in {
  options.services.mission-control = {
    enable = mkEnableOption "Mission Control system manager";
    
    port = mkOption {
      type = types.port;
      default = 3000;
      description = "Port to run Mission Control on";
    };
    
    host = mkOption {
      type = types.str;
      default = "127.0.0.1";
      description = "Host to bind to (use 0.0.0.0 for network access)";
    };
    
    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description = "Open firewall for Mission Control";
    };
    
    ollamaIntegration = mkOption {
      type = types.bool;
      default = true;
      description = "Enable Ollama integration";
    };
    
    gpuMonitoring = mkOption {
      type = types.bool;
      default = true;
      description = "Enable NVIDIA GPU monitoring";
    };
    
    systemMetrics = mkOption {
      type = types.bool;
      default = true;
      description = "Enable real-time system metrics collection";
    };
    
    metricsInterval = mkOption {
      type = types.int;
      default = 60;
      description = "Metrics collection interval in seconds";
    };
    
    dataDir = mkOption {
      type = types.path;
      default = "/var/lib/mission-control";
      description = "Data directory for Mission Control";
    };
    
    user = mkOption {
      type = types.str;
      default = "mission-control";
      description = "User to run Mission Control as";
    };
    
    group = mkOption {
      type = types.str;
      default = "mission-control";
      description = "Group to run Mission Control as";
    };
  };

  config = mkIf cfg.enable {
    # Create system user and group
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      home = cfg.dataDir;
      createHome = true;
      description = "Mission Control service user";
    };
    
    users.groups.${cfg.group} = {};

    # Required packages
    environment.systemPackages = with pkgs; [
      # Monitoring tools
      procps
      lsof
      htop
      btop
      nvtopPackages.nvidia
      iftop
      nethogs
      iotop
      ncdu
      
      # System info
      neofetch
      lshw
      dmidecode
      
      # Development
      nodejs_20
      git
      jq
      
      # AI/LLM
      ollama
    ];

    # Python environment for metrics daemon
    systemd.services.mission-control-metrics = mkIf cfg.systemMetrics {
      description = "Mission Control Metrics Collector";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      
      environment = {
        DATA_DIR = cfg.dataDir;
        METRICS_INTERVAL = toString cfg.metricsInterval;
        OLLAMA_HOST = "http://localhost:11434";
        PROC_PATH = "/proc";
        SYS_PATH = "/sys";
      };
      
      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${pkgs.python312.withPackages (ps: with ps; [ psutil gpustat ])}/bin/python ${./system-daemon.py}";
        Restart = "always";
        RestartSec = "10";
        WorkingDirectory = cfg.dataDir;
        
        # Security hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        ReadWritePaths = [ cfg.dataDir ];
        ReadOnlyPaths = [ "/proc" "/sys" ];
        
        # Capabilities for monitoring
        AmbientCapabilities = [ "CAP_SYS_PTRACE" ];
      };
    };

    # Main web service (run in development for now)
    # In production, this would be a proper built package
    systemd.services.mission-control = {
      description = "Mission Control - NixOS System Manager";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" "mission-control-metrics.service" ]
        ++ optional cfg.ollamaIntegration "ollama.service";
      wants = optional cfg.ollamaIntegration "ollama.service";
      
      environment = {
        NODE_ENV = "production";
        PORT = toString cfg.port;
        HOST = cfg.host;
        DATA_DIR = cfg.dataDir;
        OLLAMA_HOST = "http://localhost:11434";
        ENABLE_GPU_MONITORING = boolToString cfg.gpuMonitoring;
        ENABLE_SYSTEM_METRICS = boolToString cfg.systemMetrics;
      };
      
      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        WorkingDirectory = cfg.dataDir;
        # Development mode - in production use built package
        ExecStart = "${pkgs.nodejs_20}/bin/npm run start";
        Restart = "always";
        RestartSec = "5";
        
        # Security
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        ReadWritePaths = [ cfg.dataDir ];
        ReadOnlyPaths = [ "/proc" "/sys" ];
      };
    };

    # Firewall
    networking.firewall.allowedTCPPorts = mkIf cfg.openFirewall [ cfg.port ];

    # Ensure Ollama is available if integration is enabled
    services.ollama = mkIf cfg.ollamaIntegration {
      enable = mkDefault true;
      host = "0.0.0.0";
      port = 11434;
    };
  };
}
