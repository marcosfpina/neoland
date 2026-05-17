{ config, lib, ... }:

let
  cfg = config.services.neoland-llm-suite;
  gatewayUrl = "http://${cfg.host}:${toString cfg.gatewayPort}";
  mlOpsUrl = "http://${cfg.host}:${toString cfg.mlOpsPort}";
in
{
  options.services.neoland-llm-suite = {
    enable = lib.mkEnableOption "the integrated Neoland + SecureLLM + ML-Ops stack";

    host = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Loopback bind address shared by the internal LLM stack.";
    };

    gatewayPort = lib.mkOption {
      type = lib.types.port;
      default = 8080;
      description = "SecureLLM Bridge API port.";
    };

    mlOpsPort = lib.mkOption {
      type = lib.types.port;
      default = 8083;
      description = "ML-Ops API port.";
    };

    llamacppUrl = lib.mkOption {
      type = lib.types.str;
      default = "http://127.0.0.1:8081";
      description = "Upstream llama.cpp URL consumed by the ML-Ops bridge.";
    };

    vllmUrl = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "Optional upstream vLLM URL consumed by the ML-Ops bridge.";
    };

    dspyUrl = lib.mkOption {
      type = lib.types.str;
      default = "http://127.0.0.1:8001";
      description = "DSPy pipeline URL exposed to the Neoland control plane.";
    };

    neolandEnvironmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Optional EnvironmentFile for the Neoland control plane.";
    };

    securellmEnvironmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Optional EnvironmentFile for the SecureLLM gateway.";
    };

    mlOpsEnvironmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Optional EnvironmentFile for the ML-Ops bridge.";
    };
  };

  config = lib.mkIf cfg.enable {
    services.ml-ops-api = {
      enable = lib.mkDefault true;
      host = lib.mkDefault cfg.host;
      port = lib.mkDefault cfg.mlOpsPort;
      llamacppUrl = lib.mkDefault cfg.llamacppUrl;
      vllmUrl = lib.mkDefault cfg.vllmUrl;
      environmentFile = lib.mkDefault cfg.mlOpsEnvironmentFile;
    };

    services.securellm-bridge-api = {
      enable = lib.mkDefault true;
      host = lib.mkDefault cfg.host;
      port = lib.mkDefault cfg.gatewayPort;
      environmentFile = lib.mkDefault cfg.securellmEnvironmentFile;
      mlOps.enable = lib.mkDefault true;
      mlOps.apiUrl = lib.mkDefault mlOpsUrl;
      llamacpp.enable = lib.mkDefault false;
    };

    services.neoland = {
      enable = lib.mkDefault true;
      dspyUrl = lib.mkDefault cfg.dspyUrl;
      environmentFile = lib.mkDefault cfg.neolandEnvironmentFile;
      extraEnvironment = lib.mkMerge [
        {
          NEOLAND_ML_API_URL = gatewayUrl;
          ML_OPS_API_URL = mlOpsUrl;
          LLAMACPP_URL = cfg.llamacppUrl;
        }
        (lib.mkIf (cfg.vllmUrl != "") { VLLM_URL = cfg.vllmUrl; })
      ];
    };

    systemd.services.neoland.after = [ "securellm-bridge-api.service" "ml-ops-api.service" ];
    systemd.services.neoland.wants = [ "securellm-bridge-api.service" "ml-ops-api.service" ];

    systemd.services.securellm-bridge-api.after = [ "ml-ops-api.service" ];
    systemd.services.securellm-bridge-api.wants = [ "ml-ops-api.service" ];
  };
}
