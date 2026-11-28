use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const JOURNAL_VERSION: u32 = 1;
pub const JOURNAL_FILENAME: &str = "tlc-ks-journal.v1.json";
pub const TLC_PROVIDER_KEY: &str = "{A7C3E1F0-4B2D-4E9A-9C1D-8F6B2A5E3D10}";
pub const TLC_SUBLAYER_KEY: &str = "{B8D4F2E1-5C3E-4F0B-AD2E-9A7C1B4D6F20}";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KsState {
    Disabled,
    Preparing,
    Armed,
    Switching,
    Releasing,
    BlockedWaitingForTunnel,
    Reconciling,
}

impl Default for KsState {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelEndpoint {
    pub host: String,
    pub port: u16,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KsJournal {
    pub version: u32,
    pub state: KsState,
    pub generation: u64,
    pub endpoint: Option<TunnelEndpoint>,
    pub previous_endpoint: Option<TunnelEndpoint>,
    pub filter_ids: Vec<String>,
    pub interface_name: Option<String>,
    pub updated_unix: u64,
    pub filters_verified: bool,
}

impl KsJournal {
    pub fn new() -> Self {
        Self {
            version: JOURNAL_VERSION,
            state: KsState::Disabled,
            generation: 0,
            endpoint: None,
            previous_endpoint: None,
            filter_ids: Vec::new(),
            interface_name: Some("TLC-TUN".into()),
            updated_unix: now_unix(),
            filters_verified: false,
        }
    }

    pub fn touch(&mut self) {
        self.updated_unix = now_unix();
    }

    pub fn path(config_dir: &Path) -> PathBuf {
        config_dir.join(JOURNAL_FILENAME)
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let raw = fs::read_to_string(path)
            .map_err(|e| format!("خواندن ژورنال Kill Switch ناموفق: {e}"))?;
        let mut j: KsJournal = serde_json::from_str(&raw)
            .map_err(|e| format!("ژورنال Kill Switch خراب است: {e}"))?;
        if j.version != JOURNAL_VERSION {
            return Err("نسخه ژورنال Kill Switch پشتیبانی نمی‌شود".into());
        }
        if matches!(
            j.state,
            KsState::Armed | KsState::Switching | KsState::Preparing
        ) {
            j.state = KsState::Reconciling;
            j.filters_verified = false;
        }
        Ok(j)
    }

    pub fn persist(&mut self, path: &Path) -> Result<(), String> {
        self.touch();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("ایجاد پوشه پیکربندی: {e}"))?;
        }
        let tmp = path.with_extension("tmp");
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("سریال‌سازی ژورنال: {e}"))?;
        fs::write(&tmp, data.as_bytes()).map_err(|e| format!("نوشتن موقت ژورنال: {e}"))?;
        fs::rename(&tmp, path).map_err(|e| format!("جایگزینی اتمیک ژورنال: {e}"))?;
        Ok(())
    }

    pub fn clear_file(path: &Path) {
        let _ = fs::remove_file(path);
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
