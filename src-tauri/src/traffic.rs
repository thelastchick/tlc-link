//! Live traffic counters via sing-box Clash API (loopback only).
//!
//! Xray does not expose an equivalent lightweight local API in this build;
//! callers receive `available: false` instead of inventing numbers.

use crate::network::{CLASH_API_HOST, CLASH_API_PORT, CLASH_API_SECRET};
use serde::Serialize;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct TrafficSnapshot {
    /// False when core is not sing-box, API unreachable, or process not running.
    pub available: bool,
    /// Cumulative bytes since the core started (Clash API totals).
    pub upload_total: u64,
    pub download_total: u64,
    pub message: String,
}

pub fn snapshot(core_is_sing_box: bool, process_running: bool) -> TrafficSnapshot {
    if !process_running {
        return TrafficSnapshot {
            available: false,
            upload_total: 0,
            download_total: 0,
            message: "core not running".into(),
        };
    }
    if !core_is_sing_box {
        return TrafficSnapshot {
            available: false,
            upload_total: 0,
            download_total: 0,
            message: "traffic stats require sing-box in this build".into(),
        };
    }
    match fetch_connections_totals() {
        Ok((up, down)) => TrafficSnapshot {
            available: true,
            upload_total: up,
            download_total: down,
            message: "ok".into(),
        },
        Err(e) => TrafficSnapshot {
            available: false,
            upload_total: 0,
            download_total: 0,
            message: e,
        },
    }
}

fn fetch_connections_totals() -> Result<(u64, u64), String> {
    let addr = format!("{CLASH_API_HOST}:{CLASH_API_PORT}");
    let mut stream = TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|e| format!("bad clash api address: {e}"))?,
        Duration::from_millis(400),
    )
    .map_err(|e| format!("clash api connect: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_millis(600)))
        .ok();
    stream
        .set_write_timeout(Some(Duration::from_millis(400)))
        .ok();

    let req = format!(
        "GET /connections HTTP/1.1\r\nHost: {CLASH_API_HOST}:{CLASH_API_PORT}\r\nAuthorization: Bearer {CLASH_API_SECRET}\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("clash api write: {e}"))?;

    let mut buf = Vec::with_capacity(4096);
    stream
        .read_to_end(&mut buf)
        .map_err(|e| format!("clash api read: {e}"))?;
    let text = String::from_utf8_lossy(&buf);
    let body = text
        .split("\r\n\r\n")
        .nth(1)
        .ok_or_else(|| "clash api: empty body".to_string())?;
    // Status line check
    if !text.starts_with("HTTP/1.1 200") && !text.starts_with("HTTP/1.0 200") {
        let status = text.lines().next().unwrap_or("no status");
        return Err(format!("clash api status: {status}"));
    }
    let v: serde_json::Value =
        serde_json::from_str(body.trim()).map_err(|e| format!("clash api json: {e}"))?;
    let up = v
        .get("uploadTotal")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    let down = v
        .get("downloadTotal")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    Ok((up, down))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_reports_unavailable_when_not_running() {
        let s = snapshot(true, false);
        assert!(!s.available);
    }

    #[test]
    fn snapshot_reports_unavailable_for_xray() {
        let s = snapshot(false, true);
        assert!(!s.available);
        assert!(s.message.contains("sing-box"));
    }
}
