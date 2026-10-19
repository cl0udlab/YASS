import type { Server } from "./types";

export const SERIES_COLORS = {
  cpu: "#3b82f6",
  mem: "#f59e0b",
  disk: "#10b981",
  rx: "#3b82f6",
  tx: "#ef4444",
} as const;

export function formatBytes(bytes: number | null | undefined): string {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return unitIndex === 0
    ? `${bytes} ${units[unitIndex]}`
    : `${size.toFixed(2)} ${units[unitIndex]}`;
}

export function formatUptime(seconds: number | null | undefined): string {
  if (!seconds) return "0 秒";

  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  const result: string[] = [];
  if (days > 0) result.push(`${days} 天`);
  if (hours > 0) result.push(`${hours} 小時`);
  if (minutes > 0) result.push(`${minutes} 分鐘`);
  if (secs > 0 && result.length === 0) result.push(`${secs} 秒`);

  return result.slice(0, 2).join(" ");
}

export function formatTime(value: string | number | Date): string {
  return new Date(value).toLocaleString();
}

export function cpuUsage(server: Server): number {
  const cpus = server.cpu_info?.cpu_info;
  if (!cpus?.length) return 0;
  return cpus.reduce((sum, cpu) => sum + cpu.cpu_usage, 0) / cpus.length;
}

export function memoryUsage(server: Server): number {
  const m = server.memory_info;
  if (!m?.total_memory) return 0;
  return (m.used_memory / m.total_memory) * 100;
}

export function displayName(server: Server): string {
  return server.name || server.pc_name;
}
