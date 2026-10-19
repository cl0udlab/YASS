// 與 server/src/models.rs 的 JSON 結構對應

export interface SystemInfo {
  os_name: string;
  os_version: string;
  kernel_version: string;
  cpu_name: string;
  uptime: number;
  process_count: number;
}

export interface MemoryInfo {
  total_memory: number;
  used_memory: number;
  total_swap: number;
  used_swap: number;
}

export interface Network {
  interface_name: string;
  received_bytes: number;
  transmitted_bytes: number;
}

export interface Disk {
  name: string;
  total_space: number;
  available_space: number;
  used_space: number;
  file_system: string;
}

export interface OneCpuInfo {
  cpu_name: string;
  cpu_usage: number;
}

export interface Server {
  id: string;
  name: string;
  created_at: string;
  pc_name: string;
  last_seen: string;
  online: boolean;
  ip_address?: string | null;
  system_info: SystemInfo | null;
  memory_info: MemoryInfo | null;
  cpu_info: { cpu_info: OneCpuInfo[] } | null;
  disks_info: { disksinfo: Disk[] } | null;
  network_info: { network_info: Network[] } | null;
}

export interface RegistrationInfo {
  id: string;
  name: string;
  token: string;
  panel_url: string;
  install_command: string;
}

export interface HistoryPoint {
  ts: number;
  cpu: number;
  mem: number;
  disk: number;
  rx_rate: number;
  tx_rate: number;
}

export interface TrafficDay {
  date: string;
  rx_bytes: number;
  tx_bytes: number;
}

export interface TrafficResponse {
  days: TrafficDay[];
  total_rx: number;
  total_tx: number;
}

export type RankMetric = "cpu" | "mem" | "disk" | "traffic" | "rx" | "tx";

export interface RankRow {
  id: string;
  name: string;
  online: boolean;
  value: number;
}

export type HistoryRange = "1h" | "6h" | "24h" | "7d" | "30d";
