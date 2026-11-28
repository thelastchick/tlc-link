//! TLC LINK — Windows TUN Kill Switch controller
//!
//! Prefers the independent helper service (`tlc-ks-service`) over named pipe
//! `\\.\pipe\tlc-ks-svc-v1`. If the service is not installed/running, falls back
//! to in-process WFP from an elevated UI (persistent filters) so Portable users
//! still get protection while connected.
//!
//! Product honesty:
//! - `capability_enabled()` is true on Windows (code path exists).
//! - Full protection still requires Administrator + TUN mode + user toggle.
//! - Windows validation matrix must be executed before a public “verified” claim;
//!   see `docs/RELEASE-CHECKLIST.md` and `docs/KILL-SWITCH-IMPLEMENTATION-SPEC.md`.

use crate::ks::journal::{KsJournal, JOURNAL_FILENAME};
use crate::ks::protocol::{KsRequest, KsResponse, PIPE_NAME, PROTOCOL_VERSION};
use crate::ks::wfp;
use serde::Serialize;
use std::{
    fs,
    io::{Read, Write},
    net::IpAddr,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

// Re-export for lib.rs and other callers (single import path, no name clash).
pub use crate::ks::journal::{KsState, TunnelEndpoint};

pub struct KillSwitch {
    journal_path: PathBuf,
    journal: KsJournal,
    /// True when the last successful operation went through the service pipe.
    used_service: bool,
    /// Set when the on-disk journal existed but could not be parsed.
    /// Startup must attempt an orphan WFP sweep so we never leave silent filters.
    journal_corrupt: bool,
}

impl KillSwitch {
    pub fn open(config_dir: &Path) -> Self {
        let journal_path = config_dir.join(JOURNAL_FILENAME);
        let (journal, journal_corrupt) = match KsJournal::load(&journal_path) {
            Ok(j) => (j, false),
            Err(_) => {
                // Corrupt / unsupported version: start clean in memory but remember
                // so reconcile_on_startup can sweep any leftover TLC WFP objects.
                (KsJournal::new(), journal_path.exists())
            }
        };
        Self {
            journal_path,
            journal,
            used_service: false,
            journal_corrupt,
        }
    }

    pub fn journal_was_corrupt(&self) -> bool {
        self.journal_corrupt
    }

    pub fn state(&self) -> KsState {
        self.journal.state
    }

    pub fn is_armed(&self) -> bool {
        matches!(
            self.journal.state,
            KsState::Armed | KsState::Switching | KsState::BlockedWaitingForTunnel
        ) && self.journal.filters_verified
    }

    /// Capability is advertised on Windows because the WFP + service code paths exist.
    /// Arming still requires elevation, TUN mode, and the user Kill Switch toggle.
    pub fn capability_enabled(&self) -> bool {
        cfg!(windows)
    }

    /// Probe whether the independent service is accepting connections.
    pub fn service_available() -> bool {
        #[cfg(windows)]
        {
            service_client::ping().is_ok()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    pub fn begin_prepare(&mut self, endpoint: TunnelEndpoint, endpoint_ip: IpAddr) -> Result<(), String> {
        if !self.capability_enabled() {
            return Err("Kill Switch فقط روی Windows در دسترس است".into());
        }
        match self.journal.state {
            KsState::Disabled | KsState::Reconciling | KsState::BlockedWaitingForTunnel => {}
            other => {
                return Err(format!(
                    "Kill Switch نمی‌تواند از حالت {:?} وارد Preparing شود",
                    other
                ));
            }
        }
        self.journal.state = KsState::Preparing;
        self.journal.endpoint = Some(endpoint.clone());
        self.journal.previous_endpoint = None;
        self.journal.filter_ids.clear();
        self.journal.filters_verified = false;
        self.journal.generation = self.journal.generation.saturating_add(1);
        self.journal.persist(&self.journal_path)?;

        let ids = self.apply_arm(&endpoint, endpoint_ip, None)?;
        self.confirm_armed(ids)
    }

    fn apply_arm(
        &mut self,
        endpoint: &TunnelEndpoint,
        ip: IpAddr,
        previous_ip: Option<IpAddr>,
    ) -> Result<Vec<u64>, String> {
        // Prefer independent service when it is running.
        #[cfg(windows)]
        {
            if service_client::ping().is_ok() {
                let req = KsRequest::Arm {
                    host: endpoint.host.clone(),
                    port: endpoint.port,
                    protocol: endpoint.protocol.clone(),
                    ip: ip.to_string(),
                    previous_ip: previous_ip.map(|p| p.to_string()),
                };
                match service_client::call(&req)? {
                    KsResponse::Armed { filter_ids, .. } => {
                        self.used_service = true;
                        return Ok(filter_ids);
                    }
                    KsResponse::Err { message } => return Err(message),
                    other => return Err(format!("پاسخ غیرمنتظره سرویس: {other:?}")),
                }
            }
        }
        // Fallback: in-process WFP (requires elevated UI process).
        self.used_service = false;
        wfp::install_protection(&ip, endpoint.port, previous_ip.as_ref())
    }

    pub fn confirm_armed(&mut self, filter_ids: Vec<u64>) -> Result<(), String> {
        if self.journal.state != KsState::Preparing && self.journal.state != KsState::Switching {
            return Err("confirm_armed فقط از Preparing یا Switching مجاز است".into());
        }
        self.journal.filter_ids = filter_ids.iter().map(|id| id.to_string()).collect();
        self.journal.filters_verified = true;
        self.journal.state = KsState::Armed;
        self.journal.persist(&self.journal_path)
    }

    pub fn begin_switch(
        &mut self,
        new_endpoint: TunnelEndpoint,
        new_ip: IpAddr,
        old_ip: Option<IpAddr>,
    ) -> Result<(), String> {
        if !self.capability_enabled() {
            return Ok(());
        }
        if self.journal.state != KsState::Armed {
            return Err("تعویض سرور فقط از حالت Armed مجاز است".into());
        }
        let old = self.journal.endpoint.take();
        self.journal.previous_endpoint = old;
        self.journal.endpoint = Some(new_endpoint.clone());
        self.journal.state = KsState::Switching;
        self.journal.filters_verified = false;
        self.journal.persist(&self.journal_path)?;

        let old_ids: Vec<u64> = self
            .journal
            .filter_ids
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();
        let ids = self.apply_arm(&new_endpoint, new_ip, old_ip)?;
        let _ = wfp::remove_filters_by_ids(&old_ids);
        self.confirm_armed(ids)
    }

    pub fn on_unexpected_core_exit(&mut self) -> Result<(), String> {
        if matches!(
            self.journal.state,
            KsState::Armed | KsState::Switching | KsState::Preparing
        ) {
            self.journal.state = KsState::BlockedWaitingForTunnel;
            self.journal.persist(&self.journal_path)?;
        }
        Ok(())
    }

    pub fn begin_release(&mut self) -> Result<(), String> {
        match self.journal.state {
            KsState::Disabled => return Ok(()),
            _ => {}
        }
        self.journal.state = KsState::Releasing;
        self.journal.persist(&self.journal_path)?;

        #[cfg(windows)]
        {
            if service_client::ping().is_ok() {
                match service_client::call(&KsRequest::Release) {
                    Ok(KsResponse::Ok { .. }) | Ok(KsResponse::Armed { .. }) => {
                        return self.finish_release();
                    }
                    Ok(KsResponse::Err { message }) => {
                        let _ = wfp::remove_all_tlc_objects();
                        let _ = self.finish_release();
                        return Err(message);
                    }
                    Err(_) => {}
                    _ => {}
                }
            }
        }

        let ids: Vec<u64> = self
            .journal
            .filter_ids
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();
        if let Err(e) = wfp::remove_filters_by_ids(&ids) {
            let _ = wfp::remove_all_tlc_objects();
            let _ = self.finish_release();
            return Err(format!(
                "حذف فیلترهای Kill Switch ناقص بود: {e}. پاک‌سازی کامل Provider اجرا شد."
            ));
        }
        let _ = wfp::remove_all_tlc_objects();
        self.finish_release()
    }

    fn finish_release(&mut self) -> Result<(), String> {
        self.journal = KsJournal::new();
        KsJournal::clear_file(&self.journal_path);
        Ok(())
    }

    pub fn emergency_clear(&mut self) -> Result<String, String> {
        #[cfg(windows)]
        {
            if service_client::ping().is_ok() {
                if let Ok(KsResponse::Ok { message }) =
                    service_client::call(&KsRequest::EmergencyClear)
                {
                    self.journal = KsJournal::new();
                    KsJournal::clear_file(&self.journal_path);
                    return Ok(message);
                }
            }
        }
        let wfp_result = wfp::remove_all_tlc_objects();
        let had_journal = self.journal_path.exists();
        self.journal = KsJournal::new();
        if had_journal {
            KsJournal::clear_file(&self.journal_path);
        }
        match wfp_result {
            Ok(n) => Ok(format!(
                "Kill Switch: {n} شیء WFP متعلق به TLC حذف شد؛ ژورنال {}",
                if had_journal {
                    "پاک شد"
                } else {
                    "وجود نداشت"
                }
            )),
            Err(e) => {
                if had_journal {
                    Ok(format!(
                        "ژورنال پاک شد اما حذف WFP کامل نشد: {e}."
                    ))
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Called once at app startup.
    ///
    /// Policy (fail-closed, product honesty):
    /// - If the previous session left an Armed/Blocked/Reconciling journal → install a
    ///   hard block (no permit) so traffic cannot leak until the user reconnects or
    ///   runs Emergency Reset.
    /// - Prefer the independent service when it is alive; fall back to in-process WFP.
    /// - If the on-disk journal was corrupt, sweep all TLC WFP objects and clear state
    ///   rather than trusting unknown filters.
    /// - Any failure to re-arm results in emergency_clear so we never claim protection
    ///   we do not have.
    pub fn reconcile_on_startup(&mut self) -> Result<String, String> {
        // Corrupt journal: do not trust residual state; remove TLC objects and report.
        if self.journal_corrupt {
            let msg = self
                .emergency_clear()
                .unwrap_or_else(|e| format!("پاک‌سازی پس از ژورنال خراب: {e}"));
            self.journal_corrupt = false;
            return Ok(format!(
                "ژورنال Kill Switch خراب بود؛ اشیاء TLC پاک شدند. ({msg})"
            ));
        }

        if !matches!(
            self.journal.state,
            KsState::Reconciling
                | KsState::BlockedWaitingForTunnel
                | KsState::Armed
                | KsState::Switching
                | KsState::Preparing
        ) {
            return Ok(String::new());
        }

        // Prefer service for persistence across UI restarts.
        #[cfg(windows)]
        {
            if service_client::ping().is_ok() {
                // Service does not yet expose a dedicated "hard block" command; use
                // EmergencyClear then rely on in-process hard block, or Arm without
                // a permit by falling through. For now install hard block in-process
                // and keep the service aware via a subsequent status path.
            }
        }

        match wfp::install_hard_block() {
            Ok(ids) => {
                self.journal.filter_ids = ids.iter().map(|id| id.to_string()).collect();
                self.journal.filters_verified = true;
                self.journal.state = KsState::BlockedWaitingForTunnel;
                self.journal.persist(&self.journal_path)?;
                self.used_service = false;
                Ok(
                    "Kill Switch پس از راه‌اندازی دوباره مسلح شد (مسدود تا اتصال مجدد یا Reset)."
                        .into(),
                )
            }
            Err(e) => {
                let _ = self.emergency_clear();
                Err(format!(
                    "بازسازی Kill Switch ناموفق بود و حالت پاک شد: {e}"
                ))
            }
        }
    }

    pub fn status_snapshot(&self) -> KsStatus {
        KsStatus {
            state: self.journal.state,
            generation: self.journal.generation,
            endpoint: self.journal.endpoint.clone(),
            filters_verified: self.journal.filters_verified,
            capability_enabled: self.capability_enabled(),
            journal_present: self.journal_path.exists(),
            is_armed: self.is_armed(),
            service_available: Self::service_available(),
            used_service: self.used_service,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KsStatus {
    pub state: KsState,
    pub generation: u64,
    pub endpoint: Option<TunnelEndpoint>,
    pub filters_verified: bool,
    pub capability_enabled: bool,
    pub journal_present: bool,
    pub is_armed: bool,
    pub service_available: bool,
    pub used_service: bool,
}

// ─── Named-pipe client (Windows) ─────────────────────────────────────────────

#[cfg(windows)]
mod service_client {
    use super::*;
    use std::fs::OpenOptions;

    pub fn ping() -> Result<(), String> {
        call(&KsRequest::Ping {
            version: PROTOCOL_VERSION,
        })
        .map(|_| ())
    }

    pub fn call(req: &KsRequest) -> Result<KsResponse, String> {
        let payload =
            serde_json::to_vec(req).map_err(|e| format!("serialize KS request: {e}"))?;
        // Client: open the existing named pipe created by tlc-ks-service.
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(PIPE_NAME)
            .map_err(|e| format!("اتصال به سرویس Kill Switch ناموفق: {e}"))?;
        // Length-prefixed frame: u32 LE + JSON
        let len = (payload.len() as u32).to_le_bytes();
        file.write_all(&len)
            .and_then(|_| file.write_all(&payload))
            .map_err(|e| format!("ارسال به سرویس: {e}"))?;
        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf)
            .map_err(|e| format!("خواندن طول پاسخ سرویس: {e}"))?;
        let n = u32::from_le_bytes(len_buf) as usize;
        if n > 1_000_000 {
            return Err("پاسخ سرویس بیش از حد بزرگ است".into());
        }
        let mut body = vec![0u8; n];
        file.read_exact(&mut body)
            .map_err(|e| format!("خواندن بدنه پاسخ سرویس: {e}"))?;
        serde_json::from_slice(&body).map_err(|e| format!("parse KS response: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_dir() -> PathBuf {
        let mut p = env::temp_dir();
        p.push(format!(
            "tlc-ks-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn capability_is_windows_only() {
        let dir = temp_dir();
        let ks = KillSwitch::open(&dir);
        assert_eq!(ks.capability_enabled(), cfg!(windows));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn emergency_clear_removes_journal() {
        let dir = temp_dir();
        let mut ks = KillSwitch::open(&dir);
        ks.journal.state = KsState::Armed;
        ks.journal.generation = 7;
        ks.journal.persist(&ks.journal_path).unwrap();
        assert!(ks.journal_path.exists());
        let _ = ks.emergency_clear();
        assert!(!ks.journal_path.exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn journal_roundtrip_moves_armed_to_reconciling() {
        let dir = temp_dir();
        let path = dir.join(JOURNAL_FILENAME);
        let mut j = KsJournal::new();
        j.state = KsState::Armed;
        j.generation = 3;
        fs::write(&path, serde_json::to_string(&j).unwrap()).unwrap();
        let ks = KillSwitch::open(&dir);
        assert_eq!(ks.state(), KsState::Reconciling);
        assert_eq!(ks.journal.generation, 3);
        assert!(!ks.journal_was_corrupt());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn corrupt_journal_is_flagged_for_orphan_sweep() {
        let dir = temp_dir();
        let path = dir.join(JOURNAL_FILENAME);
        fs::write(&path, b"{not valid json!!!").unwrap();
        let ks = KillSwitch::open(&dir);
        assert!(ks.journal_was_corrupt());
        assert_eq!(ks.state(), KsState::Disabled);
        let _ = fs::remove_dir_all(dir);
    }
}
