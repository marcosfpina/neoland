# Mission Control - NixOS Integration Guide

Enterprise-grade system manager for NixOS with real-time monitoring, LLM laboratory, and DevOps automation.

## Quick Start

### 1. Add to your flake.nix

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    mission-control.url = "github:your-username/mission-control";
  };

  outputs = { self, nixpkgs, mission-control }: {
    nixosConfigurations.your-hostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./configuration.nix
        mission-control.nixosModules.default
        {
          services.mission-control = {
            enable = true;
            port = 3000;
            openFirewall = false;  # Set to true for network access
            ollamaIntegration = true;
            gpuMonitoring = true;
            systemMetrics = true;
          };
        }
      ];
    };
  };
}
