use super::CoreKind;
use crate::errors::AppError;
use serde::Serialize;
use std::{fs::{self, File, OpenOptions}, io::Write, path::PathBuf, process::{Child, Command, Stdio}, sync::atomic::{AtomicU64, Ordering}, thread, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Hides the console window of the core process on Windows.
fn hidden(mut cmd: Command) -> Command {
    #[cfg(windows)]
    { cmd.creation_flags(0x0800_0000); }
    cmd
}

#[cfg(debug_assertions)]
fn dev_binary(kind: CoreKind) -> Option<PathBuf> {
    Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries").join(kind.name()).join(kind.executable()))
}
#[cfg(not(debug_assertions))]
fn dev_binary(_kind: CoreKind) -> Option<PathBuf> { None }

static CONFIG_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Serialize)]
pub struct CoreStatus {
    pub process_running: bool,
    pub core: Option<String>,
    pub pid: Option<u32>,
    pub detail: String,
    /// True when the core is inside a kill-on-close Job Object (dies with TLC LINK even on a hard kill).
    pub orphan_protection: bool,
}

pub struct CoreManager {
    resource_dir: PathBuf,
    config_dir: PathBuf,
    child: Option<Child>,
    active: Option<CoreKind>,
    active_config: Option<PathBuf>,
    active_image_path: Option<String>,
    job_attached: bool,
}

impl Default for CoreManager {
    fn default() -> Self {
        Self { resource_dir: PathBuf::new(), config_dir: std::env::temp_dir().join("tlc-link"), child: None, active: None, active_config: None, active_image_path: None, job_attached: false }
    }
}

impl CoreManager {
    pub fn new(resource_dir: PathBuf) -> Self {
        let mut manager = Self::default();
        manager.resource_dir = resource_dir;
        manager
    }
    pub fn set_paths(&mut self, resource_dir: PathBuf, config_dir: PathBuf) { self.resource_dir = resource_dir; self.config_dir = config_dir; }
    pub fn executable_path(&self, kind: CoreKind) -> PathBuf {
        let packaged = self.resource_dir.join("binaries").join(kind.name()).join(kind.executable());
        if packaged.is_file() { return packaged; }
        // Development fallback exists in debug builds only, so release builds never embed a source-tree path.
        dev_binary(kind).unwrap_or(packaged)
    }
    fn write_config(&self, kind: CoreKind, raw: &str) -> Result<PathBuf, AppError> {
        let value: serde_json::Value = serde_json::from_str(raw).map_err(|e| AppError::InvalidConfig(e.to_string()))?;
        if !value.is_object() { return Err(AppError::InvalidConfig("ریشه JSON باید object باشد".into())); }
        fs::create_dir_all(&self.config_dir)?;
        // Keep each validation/start attempt isolated so a running core never sees
        // its config overwritten by a validation of another profile.
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        let bytes = serde_json::to_vec_pretty(&value).map_err(|e| AppError::InvalidConfig(e.to_string()))?;
        // create_new makes filename creation exclusive: two quick imports/starts
        // cannot silently overwrite one another's runtime config.
        for _ in 0..32 {
            let sequence = CONFIG_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = self.config_dir.join(format!("{}-{}-{}-{}.json", kind.name(), std::process::id(), stamp, sequence));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    if let Err(err) = file.write_all(&bytes) {
                        let _ = fs::remove_file(&path);
                        return Err(AppError::Spawn(err));
                    }
                    return Ok(path);
                }
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(err) => return Err(AppError::Spawn(err)),
            }
        }
        Err(AppError::InvalidConfig("ساخت فایل کانفیگ موقت یکتا ممکن نشد".into()))
    }
    fn validate_config_path(&self, kind: CoreKind, path: &PathBuf) -> Result<(), AppError> {
        let exe = self.executable_path(kind);
        if !exe.is_file() { return Err(AppError::CoreMissing(exe.display().to_string())); }
        let output = hidden(Command::new(exe)).args(kind.validate_args(path.to_string_lossy().as_ref())).stdout(Stdio::piped()).stderr(Stdio::piped()).output()?;
        if output.status.success() { Ok(()) } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let message = if stderr.trim().is_empty() { stdout.trim().to_string() } else { stderr.trim().to_string() };
            Err(AppError::InvalidConfig(message))
        }
    }
    pub fn validate_json(&self, kind: CoreKind, raw: &str) -> Result<(), AppError> {
        let path = self.write_config(kind, raw)?;
        let result = self.validate_config_path(kind, &path);
        let _ = fs::remove_file(path);
        result
    }
    pub fn start(&mut self, kind: CoreKind, raw: &str) -> Result<CoreStatus, AppError> {
        // Validate the candidate before stopping an existing session. Reuse the
        // exact validated file for startup to avoid a config-write race.
        let path = self.write_config(kind, raw)?;
        if let Err(err) = self.validate_config_path(kind, &path) {
            let _ = fs::remove_file(&path);
            return Err(err);
        }
        if self.child.is_some() {
            if let Err(err) = self.stop() {
                let _ = fs::remove_file(&path);
                return Err(err);
            }
        }
        let exe = self.executable_path(kind);
        let log_path = self.config_dir.join(format!("{}-core.log", kind.name()));
        // Keep a single previous log to prevent unbounded log growth.
        let previous_log = self.config_dir.join(format!("{}-core.log.1", kind.name()));
        if log_path.exists() {
            let _ = fs::remove_file(&previous_log);
            let _ = fs::rename(&log_path, &previous_log);
        }
        let log_out = match File::create(&log_path) {
            Ok(file) => file,
            Err(err) => { let _ = fs::remove_file(&path); return Err(AppError::Spawn(err)); }
        };
        let log_err = match log_out.try_clone() {
            Ok(file) => file,
            Err(err) => { let _ = fs::remove_file(&path); return Err(AppError::Spawn(err)); }
        };
        let mut child = match hidden(Command::new(exe))
            .args(kind.run_args(path.to_string_lossy().as_ref()))
            .stdin(Stdio::null())
            .stdout(Stdio::from(log_out))
            .stderr(Stdio::from(log_err))
            .spawn() {
                Ok(child) => child,
                Err(err) => { let _ = fs::remove_file(&path); return Err(AppError::Spawn(err)); }
            };
        // Tie the core's lifetime to ours: if TLC LINK is killed, Windows terminates the core.
        let job_attached = crate::winproc::attach_to_kill_on_close_job(&child).is_ok();
        // Poll startup for a short window so quick crashes are surfaced instead of
        // being reported as a successful launch. Keep the window bounded.
        let startup_deadline = Instant::now() + Duration::from_millis(900);
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let details = fs::read_to_string(&log_path).unwrap_or_default();
                    let tail = details.lines().rev().take(24).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
                    let _ = fs::remove_file(&path);
                    return Err(AppError::InvalidConfig(format!("هسته هنگام راه‌اندازی متوقف شد ({status}). {}", tail.trim())));
                }
                Ok(None) => {}
                Err(err) => {
                    // Child::drop does not terminate a process. Explicitly kill and
                    // wait here before returning so a monitoring error cannot orphan it.
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = fs::remove_file(&path);
                    return Err(AppError::Spawn(err));
                }
            }
            if Instant::now() >= startup_deadline { break; }
            thread::sleep(Duration::from_millis(40));
        }
        let pid = child.id();
        self.child = Some(child);
        self.active = Some(kind);
        self.active_config = Some(path);
        self.active_image_path = Some(self.executable_path(kind).to_string_lossy().into_owned());
        self.job_attached = job_attached;
        Ok(CoreStatus { process_running: true, core: Some(kind.name().into()), pid: Some(pid), detail: "فرایند هسته فعال است؛ برقراری تونل و دسترسی شبکه هنوز تأیید نشده است.".into(), orphan_protection: job_attached })
    }
    pub fn stop(&mut self) -> Result<CoreStatus, AppError> {
        if let Some(mut child) = self.child.take() {
            match child.try_wait() {
                Ok(Some(_)) => { /* already exited */ }
                Ok(None) => {
                    if let Err(err) = child.kill() {
                        self.child = Some(child);
                        return Err(AppError::Spawn(err));
                    }
                    if let Err(err) = child.wait() {
                        self.child = Some(child);
                        return Err(AppError::Spawn(err));
                    }
                }
                Err(err) => { self.child = Some(child); return Err(AppError::Spawn(err)); }
            }
        }
        self.active = None;
        self.active_image_path = None;
        self.job_attached = false;
        if let Some(config) = self.active_config.take() { let _ = fs::remove_file(config); }
        Ok(self.status())
    }
    /// Full path of the running core executable (used for identity-checked orphan cleanup).
    pub fn active_image_path(&self) -> Option<String> { self.active_image_path.clone() }
    pub fn is_running(&self) -> bool { self.child.is_some() }
    pub fn active_core_name(&self) -> Option<String> { self.active.map(|c| c.name().into()) }
    pub fn status(&self) -> CoreStatus {
        CoreStatus { process_running: self.child.is_some(), core: self.active.map(|c| c.name().into()), pid: self.child.as_ref().map(Child::id), detail: if self.child.is_some() { "فرایند هسته فعال است؛ وضعیت مسیر شبکه به‌صورت مستقل باید سنجیده شود.".into() } else { "اتصال قطع است".into() }, orphan_protection: self.child.is_some() && self.job_attached }
    }
    pub fn refresh_status(&mut self) -> CoreStatus {
        let exited = match self.child.as_mut() {
            Some(child) => child.try_wait().map(|status| status.is_some()).unwrap_or(false),
            None => false,
        };
        if exited {
            self.child = None;
            self.active = None;
            self.active_image_path = None;
            self.job_attached = false;
            if let Some(config) = self.active_config.take() { let _ = fs::remove_file(config); }
        }
        self.status()
    }
}


impl Drop for CoreManager {
    fn drop(&mut self) {
        // Best-effort cleanup when the desktop app closes unexpectedly or exits.
        if let Some(mut child) = self.child.take() {
            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
        if let Some(config) = self.active_config.take() { let _ = fs::remove_file(config); }
    }
}

#[cfg(test)]
mod process_tests {
    use super::{CoreManager, CoreKind};
    use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

    fn test_dir() -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("tlc-link-test-{}-{}", std::process::id(), stamp))
    }

    #[test]
    fn runtime_config_files_are_unique_and_json_is_normalized() {
        let dir = test_dir();
        let mut manager = CoreManager::default();
        manager.set_paths(PathBuf::new(), dir.clone());
        let first = manager.write_config(CoreKind::SingBox, r#"{"log":{"level":"warn"}}"#).unwrap();
        let second = manager.write_config(CoreKind::SingBox, r#"{"log":{"level":"warn"}}"#).unwrap();
        assert_ne!(first, second);
        assert!(first.is_file() && second.is_file());
        let parsed: serde_json::Value = serde_json::from_str(&fs::read_to_string(&first).unwrap()).unwrap();
        assert!(parsed.is_object());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn runtime_config_rejects_invalid_json_and_non_object_roots() {
        let dir = test_dir();
        let mut manager = CoreManager::default();
        manager.set_paths(PathBuf::new(), dir.clone());
        assert!(manager.write_config(CoreKind::Xray, "not-json").is_err());
        assert!(manager.write_config(CoreKind::Xray, "[]").is_err());
        let _ = fs::remove_dir_all(dir);
    }
}
