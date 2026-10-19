use crate::models::SystemInfoPayload;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::sync::Mutex;

const RAW_RETENTION_SECS: i64 = 3 * 24 * 3600;
const HOURLY_RETENTION_SECS: i64 = 30 * 24 * 3600;
const DAILY_RETENTION_SECS: i64 = 365 * 24 * 3600;
const MAX_POINTS: usize = 500;

const SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;

CREATE TABLE IF NOT EXISTS metrics (
    server_id     TEXT    NOT NULL,
    ts            INTEGER NOT NULL,
    interval_secs INTEGER NOT NULL,
    cpu_usage     REAL    NOT NULL,
    mem_used      INTEGER NOT NULL,
    mem_total     INTEGER NOT NULL,
    swap_used     INTEGER NOT NULL,
    swap_total    INTEGER NOT NULL,
    disk_used     INTEGER NOT NULL,
    disk_total    INTEGER NOT NULL,
    rx_total      INTEGER NOT NULL,
    tx_total      INTEGER NOT NULL,
    rx_delta      INTEGER NOT NULL,
    tx_delta      INTEGER NOT NULL,
    uptime        INTEGER NOT NULL,
    processes     INTEGER NOT NULL,
    PRIMARY KEY (server_id, ts)
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS metrics_hourly (
    server_id TEXT    NOT NULL,
    bucket    INTEGER NOT NULL,
    samples   INTEGER NOT NULL,
    cpu_avg   REAL    NOT NULL,
    cpu_max   REAL    NOT NULL,
    mem_avg   REAL    NOT NULL,
    mem_max   REAL    NOT NULL,
    disk_avg  REAL    NOT NULL,
    rx_bytes  INTEGER NOT NULL,
    tx_bytes  INTEGER NOT NULL,
    PRIMARY KEY (server_id, bucket)
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS metrics_daily (
    server_id TEXT    NOT NULL,
    bucket    INTEGER NOT NULL,
    samples   INTEGER NOT NULL,
    cpu_avg   REAL    NOT NULL,
    cpu_max   REAL    NOT NULL,
    mem_avg   REAL    NOT NULL,
    mem_max   REAL    NOT NULL,
    disk_avg  REAL    NOT NULL,
    rx_bytes  INTEGER NOT NULL,
    tx_bytes  INTEGER NOT NULL,
    PRIMARY KEY (server_id, bucket)
) WITHOUT ROWID;
"#;

#[derive(Serialize, Debug, Clone)]
pub struct MetricPoint {
    pub ts: i64,
    pub cpu: f64,
    pub mem: f64,
    pub disk: f64,
    pub rx_rate: i64,
    pub tx_rate: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct TrafficDay {
    pub date: String,
    pub rx_bytes: i64,
    pub tx_bytes: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct Diagnostics {
    pub db_bytes: u64,
    pub wal_bytes: u64,
    pub raw_rows: i64,
    pub hourly_rows: i64,
    pub daily_rows: i64,
    pub oldest_raw: Option<i64>,
    pub tracked_servers: i64,
    pub page_count: i64,
    pub freelist_count: i64,
    pub reclaimable_bytes: i64,
}

pub struct MetricStore {
    conn: Mutex<Connection>,
    path: String,
}

impl MetricStore {
    pub fn new(path: &str) -> Self {
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(path).expect("開啟 metrics 資料庫失敗");
        conn.execute_batch(SCHEMA)
            .expect("建立 metrics schema 失敗");

        Self {
            conn: Mutex::new(conn),
            path: path.to_string(),
        }
    }

    pub fn record(&self, payload: &SystemInfoPayload) -> rusqlite::Result<()> {
        let now = chrono::Utc::now().timestamp();

        let cpu = if payload.cpu_info.cpu_info.is_empty() {
            0.0
        } else {
            payload
                .cpu_info
                .cpu_info
                .iter()
                .map(|c| c.cpu_usage as f64)
                .sum::<f64>()
                / payload.cpu_info.cpu_info.len() as f64
        };

        let (disk_used, disk_total) = payload
            .disks_info
            .disksinfo
            .iter()
            .fold((0i64, 0i64), |(u, t), d| {
                (u + d.used_space as i64, t + d.total_space as i64)
            });

        let (rx_total, tx_total) = payload
            .network_info
            .network_info
            .iter()
            .filter(|n| n.interface_name != "lo")
            .fold((0i64, 0i64), |(r, t), n| {
                (r + n.received_bytes as i64, t + n.transmitted_bytes as i64)
            });

        let conn = self.conn.lock().unwrap();

        let previous: Option<(i64, i64, i64)> = conn
            .query_row(
                "SELECT ts, rx_total, tx_total FROM metrics
                 WHERE server_id = ?1 ORDER BY ts DESC LIMIT 1",
                params![payload.server_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .ok();

        let (interval, rx_delta, tx_delta) = match previous {
            Some((prev_ts, prev_rx, prev_tx)) => (
                (now - prev_ts).max(1),
                (rx_total - prev_rx).max(0),
                (tx_total - prev_tx).max(0),
            ),
            None => (0, 0, 0),
        };

        conn.execute(
            "INSERT OR REPLACE INTO metrics (
                server_id, ts, interval_secs, cpu_usage,
                mem_used, mem_total, swap_used, swap_total,
                disk_used, disk_total, rx_total, tx_total,
                rx_delta, tx_delta, uptime, processes
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
            params![
                payload.server_id,
                now,
                interval,
                cpu,
                payload.memory_info.used_memory as i64,
                payload.memory_info.total_memory as i64,
                payload.memory_info.used_swap as i64,
                payload.memory_info.total_swap as i64,
                disk_used,
                disk_total,
                rx_total,
                tx_total,
                rx_delta,
                tx_delta,
                payload.system_info.uptime as i64,
                payload.system_info.process_count as i64,
            ],
        )?;

        Ok(())
    }

    pub fn history(&self, server_id: &str, range_secs: i64) -> rusqlite::Result<Vec<MetricPoint>> {
        let since = chrono::Utc::now().timestamp() - range_secs;
        let conn = self.conn.lock().unwrap();

        if range_secs <= RAW_RETENTION_SECS {
            let mut stmt = conn.prepare(
                "SELECT ts, cpu_usage,
                        mem_used * 100.0 / NULLIF(mem_total, 0),
                        disk_used * 100.0 / NULLIF(disk_total, 0),
                        rx_delta, tx_delta, interval_secs
                 FROM metrics
                 WHERE server_id = ?1 AND ts >= ?2
                 ORDER BY ts",
            )?;
            let rows = stmt.query_map(params![server_id, since], |row| {
                let interval: i64 = row.get(6)?;
                let divisor = interval.max(1);
                Ok(MetricPoint {
                    ts: row.get(0)?,
                    cpu: row.get(1)?,
                    mem: row.get::<_, Option<f64>>(2)?.unwrap_or(0.0),
                    disk: row.get::<_, Option<f64>>(3)?.unwrap_or(0.0),
                    rx_rate: row.get::<_, i64>(4)? / divisor,
                    tx_rate: row.get::<_, i64>(5)? / divisor,
                })
            })?;
            return Ok(downsample(rows.collect::<rusqlite::Result<Vec<_>>>()?));
        }

        let (table, bucket_secs) = if range_secs <= HOURLY_RETENTION_SECS {
            ("metrics_hourly", 3600i64)
        } else {
            ("metrics_daily", 86400i64)
        };

        let mut stmt = conn.prepare(&format!(
            "SELECT bucket, cpu_avg, mem_avg, disk_avg, rx_bytes, tx_bytes
             FROM {} WHERE server_id = ?1 AND bucket >= ?2 ORDER BY bucket",
            table
        ))?;
        let rows = stmt.query_map(params![server_id, since], |row| {
            Ok(MetricPoint {
                ts: row.get(0)?,
                cpu: row.get(1)?,
                mem: row.get(2)?,
                disk: row.get(3)?,
                rx_rate: row.get::<_, i64>(4)? / bucket_secs,
                tx_rate: row.get::<_, i64>(5)? / bucket_secs,
            })
        })?;
        Ok(downsample(rows.collect::<rusqlite::Result<Vec<_>>>()?))
    }

    pub fn traffic(&self, server_id: &str, days: i64) -> rusqlite::Result<Vec<TrafficDay>> {
        let now = chrono::Utc::now().timestamp();
        let since = now - days * 86400;
        let today = (now / 86400) * 86400;
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT bucket, SUM(rx), SUM(tx) FROM (
                 SELECT (ts / 86400) * 86400 AS bucket, rx_delta AS rx, tx_delta AS tx
                 FROM metrics WHERE server_id = ?1 AND ts >= ?3
                 UNION ALL
                 SELECT bucket, rx_bytes, tx_bytes
                 FROM metrics_daily WHERE server_id = ?1 AND bucket >= ?2 AND bucket < ?3
             ) GROUP BY bucket ORDER BY bucket",
        )?;

        let rows = stmt.query_map(params![server_id, since, today], |row| {
            let bucket: i64 = row.get(0)?;
            let date = chrono::DateTime::from_timestamp(bucket, 0)
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            Ok(TrafficDay {
                date,
                rx_bytes: row.get(1)?,
                tx_bytes: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn traffic_totals(&self, days: i64) -> rusqlite::Result<Vec<(String, i64, i64)>> {
        let now = chrono::Utc::now().timestamp();
        let since = now - days * 86400;
        let today = (now / 86400) * 86400;
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT server_id, SUM(rx), SUM(tx) FROM (
                 SELECT server_id, rx_delta AS rx, tx_delta AS tx
                 FROM metrics WHERE ts >= ?2
                 UNION ALL
                 SELECT server_id, rx_bytes, tx_bytes
                 FROM metrics_daily WHERE bucket >= ?1 AND bucket < ?2
             ) GROUP BY server_id",
        )?;

        let rows = stmt.query_map(params![since, today], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        rows.collect()
    }

    pub fn rollup(&self) -> rusqlite::Result<()> {
        let now = chrono::Utc::now().timestamp();
        let hour_start = (now / 3600) * 3600;
        let day_start = (now / 86400) * 86400;

        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO metrics_hourly
             SELECT server_id, (ts / 3600) * 3600, COUNT(*),
                    AVG(cpu_usage), MAX(cpu_usage),
                    AVG(mem_used * 100.0 / NULLIF(mem_total, 0)),
                    MAX(mem_used * 100.0 / NULLIF(mem_total, 0)),
                    AVG(disk_used * 100.0 / NULLIF(disk_total, 0)),
                    SUM(rx_delta), SUM(tx_delta)
             FROM metrics WHERE ts < ?1 AND mem_total > 0 AND disk_total > 0
             GROUP BY server_id, ts / 3600",
            params![hour_start],
        )?;

        conn.execute(
            "INSERT OR REPLACE INTO metrics_daily
             SELECT server_id, (bucket / 86400) * 86400, SUM(samples),
                    SUM(cpu_avg * samples) / SUM(samples), MAX(cpu_max),
                    SUM(mem_avg * samples) / SUM(samples), MAX(mem_max),
                    SUM(disk_avg * samples) / SUM(samples),
                    SUM(rx_bytes), SUM(tx_bytes)
             FROM metrics_hourly WHERE bucket < ?1
             GROUP BY server_id, bucket / 86400",
            params![day_start],
        )?;

        conn.execute(
            "DELETE FROM metrics WHERE ts < ?1",
            params![now - RAW_RETENTION_SECS],
        )?;
        conn.execute(
            "DELETE FROM metrics_hourly WHERE bucket < ?1",
            params![now - HOURLY_RETENTION_SECS],
        )?;
        conn.execute(
            "DELETE FROM metrics_daily WHERE bucket < ?1",
            params![now - DAILY_RETENTION_SECS],
        )?;

        Ok(())
    }

    pub fn purge_server(&self, server_id: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        for table in ["metrics", "metrics_hourly", "metrics_daily"] {
            conn.execute(
                &format!("DELETE FROM {} WHERE server_id = ?1", table),
                params![server_id],
            )?;
        }
        Ok(())
    }

    pub fn diagnostics(&self) -> rusqlite::Result<Diagnostics> {
        let conn = self.conn.lock().unwrap();

        let count = |table: &str| -> rusqlite::Result<i64> {
            conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |r| r.get(0))
        };

        let page_count: i64 = conn.query_row("PRAGMA page_count", [], |r| r.get(0))?;
        let page_size: i64 = conn.query_row("PRAGMA page_size", [], |r| r.get(0))?;
        let freelist_count: i64 = conn.query_row("PRAGMA freelist_count", [], |r| r.get(0))?;

        let file_len = |suffix: &str| {
            std::fs::metadata(format!("{}{}", self.path, suffix))
                .map(|m| m.len())
                .unwrap_or(0)
        };

        Ok(Diagnostics {
            db_bytes: file_len(""),
            wal_bytes: file_len("-wal"),
            raw_rows: count("metrics")?,
            hourly_rows: count("metrics_hourly")?,
            daily_rows: count("metrics_daily")?,
            oldest_raw: conn
                .query_row("SELECT MIN(ts) FROM metrics", [], |r| {
                    r.get::<_, Option<i64>>(0)
                })
                .unwrap_or(None),
            tracked_servers: conn.query_row(
                "SELECT COUNT(DISTINCT server_id) FROM metrics",
                [],
                |r| r.get(0),
            )?,
            page_count,
            freelist_count,
            reclaimable_bytes: freelist_count * page_size,
        })
    }

    pub fn vacuum(&self) -> rusqlite::Result<u64> {
        let conn = self.conn.lock().unwrap();
        let before = std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);
        conn.execute_batch(
            "PRAGMA wal_checkpoint(TRUNCATE); VACUUM; PRAGMA wal_checkpoint(TRUNCATE);",
        )?;
        let after = std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);
        Ok(before.saturating_sub(after))
    }
}

fn downsample(points: Vec<MetricPoint>) -> Vec<MetricPoint> {
    if points.len() <= MAX_POINTS {
        return points;
    }
    let step = points.len() / MAX_POINTS + 1;
    points.into_iter().step_by(step).collect()
}

lazy_static::lazy_static! {
    pub static ref METRIC_STORE: MetricStore = MetricStore::new("data/metrics.db");
}
