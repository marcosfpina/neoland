/**
 * Mission Control System Types
 * Enterprise-grade type definitions for NixOS integration
 */

export interface SystemMetrics {
  timestamp: string
  hostname: string
  uptime_seconds: number
  cpu: CpuInfo
  memory: MemoryInfo
  gpu: GpuInfo[]
  disks: DiskInfo[]
  network: NetworkInfo
  processes: ProcessInfo[]
  services: ServiceInfo[]
  ollama: OllamaStatus
  thermal: Record<string, ThermalZone>
  power: PowerInfo
}

export interface CpuInfo {
  cores: number
  model: string
  frequency_mhz: number
  usage_percent: number
  per_core_usage: number[]
  load_average: [number, number, number]
  governor: string
}

export interface MemoryInfo {
  total_bytes: number
  available_bytes: number
  used_bytes: number
  free_bytes: number
  cached_bytes: number
  buffers_bytes: number
  swap_total_bytes: number
  swap_free_bytes: number
  swap_used_bytes: number
  usage_percent: number
  swap_usage_percent: number
  zram_used_bytes: number
}

export interface GpuInfo {
  index: number
  name: string
  temperature_c: number
  memory_used_mb: number
  memory_total_mb: number
  utilization_percent: number
  power_draw_w: number
  power_limit_w: number
}

export interface DiskInfo {
  device: string
  mountpoint: string
  fstype: string
  total_bytes: number
  used_bytes: number
  free_bytes: number
  usage_percent: number
}

export interface NetworkInfo {
  interfaces: Record<string, NetworkInterface>
  total_rx_bytes: number
  total_tx_bytes: number
}

export interface NetworkInterface {
  rx_bytes: number
  rx_packets: number
  rx_errors: number
  tx_bytes: number
  tx_packets: number
  tx_errors: number
}

export interface ProcessInfo {
  pid: number
  name: string
  cpu_percent: number
  memory_percent: number
  status: string
  user: string
}

export interface ServiceInfo {
  name: string
  active_state: "active" | "inactive" | "failed" | "activating" | "deactivating" | "unknown"
  sub_state: string
  pid: number
  memory_bytes: number
}

export interface OllamaStatus {
  running: boolean
  models: OllamaModel[]
  loaded_models: LoadedModel[]
  queue_size: number
}

export interface OllamaModel {
  name: string
  size: number
  modified_at: string
}

export interface LoadedModel {
  name: string
  size: number
  vram_size: number
}

export interface ThermalZone {
  type: string
  temperature_c: number
}

export interface PowerInfo {
  ac_online: boolean
  battery_present: boolean
  battery_percent: number
  battery_status: string
  power_profile: string
}

export interface MetricsHistory {
  timestamp: string
  cpu_usage: number
  memory_usage: number
  gpu_usage: number
  gpu_memory: number
  swap_usage: number
}

// Agent Types with System Integration
export interface SystemAgent {
  id: string
  name: string
  codename: string
  role: "orchestrator" | "monitor" | "executor" | "analyst" | "reporter" | "optimizer"
  status: "idle" | "working" | "paused" | "error"
  specialization: string[]
  capabilities: string[]
  metrics: AgentMetrics
  systemAccess: SystemAccessLevel
}

export interface AgentMetrics {
  tasksCompleted: number
  successRate: number
  avgResponseTime: number
  xp: number
  level: number
  streak: number
  efficiency: number
}

export interface SystemAccessLevel {
  canReadProc: boolean
  canReadSys: boolean
  canControlServices: boolean
  canModifyConfig: boolean
  canAccessGpu: boolean
  canAccessNetwork: boolean
}

// NixOS Configuration Types
export interface NixOSConfig {
  hostname: string
  stateVersion: string
  bootLoader: BootLoaderConfig
  networking: NetworkingConfig
  services: ServicesConfig
  hardware: HardwareConfig
  users: UsersConfig
  systemPackages: string[]
}

export interface BootLoaderConfig {
  systemdBoot: boolean
  efiCanTouchVariables: boolean
  cleanTmpDir: boolean
}

export interface NetworkingConfig {
  hostName: string
  networkManager: boolean
  firewall: {
    enable: boolean
    allowedTCPPorts: number[]
    allowedUDPPorts: number[]
  }
}

export interface ServicesConfig {
  xserver: XServerConfig
  pipewire: { enable: boolean }
  ollama: OllamaServiceConfig
  tlp: TlpConfig
  earlyoom: EarlyOomConfig
}

export interface XServerConfig {
  enable: boolean
  layout: string
  displayManager: string
  desktopManager: string
  videoDrivers: string[]
}

export interface OllamaServiceConfig {
  enable: boolean
  host: string
  maxLoadedModels: number
  maxQueue: number
  cudaEnabled: boolean
}

export interface TlpConfig {
  enable: boolean
  cpuGovernorOnAC: string
  cpuGovernorOnBat: string
  cpuBoostOnAC: boolean
  cpuBoostOnBat: boolean
}

export interface EarlyOomConfig {
  enable: boolean
  freeMemThreshold: number
  freeSwapThreshold: number
  enableNotifications: boolean
}

export interface HardwareConfig {
  nvidia: {
    enable: boolean
    modesetting: boolean
    powerManagement: boolean
    open: boolean
    package: string
  }
  opengl: boolean
}

export interface UsersConfig {
  users: Record<string, UserConfig>
}

export interface UserConfig {
  isNormalUser: boolean
  extraGroups: string[]
}
