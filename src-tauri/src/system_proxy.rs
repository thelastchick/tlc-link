//! Windows System Proxy (WinINet) with exact snapshot / restore and crash recovery.
//!
//! Safety model
//! 1. The previous values (ProxyEnable, ProxyServer, ProxyOverride, AutoConfigURL, AutoDetect)
//!    are written to `system-proxy-state.json` (atomic write + fsync) BEFORE the registry changes.
//!    "Value absent" (`None`) is distinct from "value is 0 / empty string".
//! 2. The state file exists exactly while TLC LINK owns the proxy settings. It is deleted only
//!    after the registry was rewritten AND re-read and matches the snapshot.
//! 3. An unreadable/corrupt state file is never treated as "no file": it is preserved, the user
//!    gets an actionable error, and no registry change is made on top of an unknown state.
//! 4. All registry/process access goes through `ProxyBackend`, so the transactional logic is unit
//!    tested with an in-memory fake; the Windows backend uses the Win32 registry API directly.
//!
//! Not compiled or run in the authoring environment (no Rust toolchain, no Windows).
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::{Path, PathBuf}};

pub const OVERRIDE: &str = "localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;172.19.*;172.20.*;172.21.*;172.22.*;172.23.*;172.24.*;172.25.*;172.26.*;172.27.*;172.28.*;172.29.*;172.30.*;172.31.*;192.168.*;<local>";
const STATE_FILE: &str = "system-proxy-state.json";
const STATE_VERSION: u32 = 2;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxySnapshot {
    pub proxy_enable: Option<u32>,
    pub proxy_server: Option<String>,
    pub proxy_override: Option<String>,
    pub auto_config_url: Option<String>,
    pub auto_detect: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProxyState {
    #[serde(default)]
    version: u32,
    snapshot: ProxySnapshot,
    server: String,
    core_pid: Option<u32>,
    /// Full path of the core executable (identity check before killing an orphan).
    #[serde(default)]
    core_image_path: Option<String>,
    /// True when no trustworthy original snapshot existed (state file was lost while our own
    /// proxy values were still in the registry). Restore then falls back to "proxy disabled".
    #[serde(default)]
    snapshot_unreliable: bool,
}

#[derive(Debug)]
enum StateRead { Missing, Valid(ProxyState), Corrupt(String) }

#[derive(Debug, PartialEq, Eq)]
pub enum RestoreOutcome { NothingToRestore, Restored { snapshot_unreliable: bool } }

/// Registry / process access. The Windows implementation is `WinBackend`.
pub trait ProxyBackend: Send + Sync {
    fn read(&self) -> Result<ProxySnapshot, String>;
    /// Points Windows at `server` (manual proxy), disabling PAC / auto-detect for the duration.
    fn set_proxy(&self, server: &str, bypass: &str) -> Result<(), String>;
    /// Writes every value back exactly; `None` means "delete the value".
    fn write(&self, snapshot: &ProxySnapshot) -> Result<(), String>;
    /// Ok(true) killed, Ok(false) process gone or not ours.
    fn kill_orphan(&self, pid: u32, image_path: &str) -> Result<bool, String>;
}

pub struct SystemProxy {
    state_path: PathBuf,
    backend: Box<dyn ProxyBackend>,
}

fn is_ours(s: &ProxySnapshot, ours: &[&str]) -> bool {
    s.proxy_enable == Some(1) && s.proxy_server.as_deref().map(|v| ours.contains(&v)).unwrap_or(false)
}

impl SystemProxy {
    pub fn new(config_dir: &Path) -> Self { Self::with_backend(config_dir, Box::new(WinBackend)) }
    pub fn with_backend(config_dir: &Path, backend: Box<dyn ProxyBackend>) -> Self {
        Self { state_path: config_dir.join(STATE_FILE), backend }
    }

    /// True while TLC LINK owns (or may own) the Windows proxy settings. A corrupt file counts.
    pub fn is_owned(&self) -> bool { self.state_path.exists() }

    /// Fails early (before anything is stopped or changed) when the state file is corrupt.
    pub fn preflight(&self) -> Result<(), String> {
        match self.read_state() { StateRead::Corrupt(e) => Err(corrupt_message(&e)), _ => Ok(()) }
    }

    /// Saves the previous settings, points Windows at `server`, then re-reads the registry to verify.
    /// On any failure the snapshot is restored and an error returned.
    pub fn apply(&self, server: &str, core_pid: Option<u32>, core_image_path: Option<&str>) -> Result<(), String> {
        if let Some(dir) = self.state_path.parent() { fs::create_dir_all(dir).map_err(|e| format!("ساخت پوشه وضعیت ممکن نشد: {e}"))?; }
        let mut state = match self.read_state() {
            // Existing ownership: keep the ORIGINAL snapshot, never snapshot our own values.
            StateRead::Valid(existing) => existing,
            StateRead::Corrupt(e) => return Err(corrupt_message(&e)),
            StateRead::Missing => {
                let current = self.backend.read().map_err(|e| format!("خواندن تنظیمات فعلی پروکسی ویندوز ناموفق بود؛ هیچ تغییری اعمال نشد: {e}"))?;
                let ours = [crate::network::system_proxy_server("sing-box"), crate::network::system_proxy_server("xray")];
                let ours_ref: Vec<&str> = ours.iter().map(String::as_str).collect();
                if is_ours(&current, &ours_ref) {
                    // Our values are already live but the state file is gone: the real original is unknown.
                    ProxyState { version: STATE_VERSION, snapshot: ProxySnapshot { proxy_enable: Some(0), ..Default::default() }, server: server.into(), core_pid: None, core_image_path: None, snapshot_unreliable: true }
                } else {
                    ProxyState { version: STATE_VERSION, snapshot: current, server: server.into(), core_pid: None, core_image_path: None, snapshot_unreliable: false }
                }
            }
        };
        state.version = STATE_VERSION;
        state.server = server.into();
        state.core_pid = core_pid;
        state.core_image_path = core_image_path.map(str::to_string);
        self.write_state(&state)?; // snapshot is durable BEFORE the registry changes

        if let Err(err) = self.backend.set_proxy(server, OVERRIDE) {
            let tail = match self.restore() { Ok(_) => String::new(), Err(e) => format!(" | بازیابی هم ناموفق بود: {e}") };
            return Err(format!("اعمال System Proxy ناموفق بود: {err}{tail}"));
        }
        match self.is_applied(server) {
            Ok(true) => Ok(()),
            other => {
                let why = match other { Err(e) => e, _ => "مقدار رجیستری پس از نوشتن مطابق انتظار نیست".into() };
                let tail = match self.restore() { Ok(_) => "تنظیمات قبلی بازگردانده شد.".to_string(), Err(e) => format!("بازیابی هم ناموفق بود: {e}") };
                Err(format!("System Proxy ویندوز تأیید نشد ({why}). {tail}"))
            }
        }
    }

    /// Restores the saved settings, re-reads the registry, and only then deletes the state file.
    pub fn restore(&self) -> Result<RestoreOutcome, String> {
        let state = match self.read_state() {
            StateRead::Missing => return Ok(RestoreOutcome::NothingToRestore),
            StateRead::Corrupt(e) => return Err(corrupt_message(&e)),
            StateRead::Valid(s) => s,
        };
        self.backend.write(&state.snapshot).map_err(|e| format!("بازیابی رجیستری ناموفق بود؛ اطلاعات برای تلاش مجدد حفظ شد: {e}"))?;
        let now = self.backend.read().map_err(|e| format!("بازخوانی رجیستری پس از بازیابی ناموفق بود؛ اطلاعات حفظ شد: {e}"))?;
        if now != state.snapshot {
            return Err("پس از بازیابی، مقادیر رجیستری با Snapshot یکسان نیست؛ فایل وضعیت برای تلاش مجدد حفظ شد.".into());
        }
        fs::remove_file(&self.state_path).map_err(|e| format!("تنظیمات بازگردانده شد اما حذف فایل وضعیت ممکن نشد: {e}"))?;
        Ok(RestoreOutcome::Restored { snapshot_unreliable: state.snapshot_unreliable })
    }

    /// Startup recovery after a crash / forced close: stop an orphaned core (identity-checked),
    /// then restore. Ok(false) = nothing was dirty.
    pub fn recover_if_dirty(&self) -> Result<bool, String> {
        let state = match self.read_state() {
            StateRead::Missing => return Ok(false),
            StateRead::Corrupt(e) => return Err(corrupt_message(&e)),
            StateRead::Valid(s) => s,
        };
        if let (Some(pid), Some(path)) = (state.core_pid, state.core_image_path.as_deref()) {
            // A core that is still alive keeps the proxy usable; killing it first is fine because
            // the registry is restored immediately afterwards.
            if let Err(e) = self.backend.kill_orphan(pid, path) { return Err(format!("توقف هسته یتیم ناموفق بود: {e}")); }
        }
        self.restore().map(|o| matches!(o, RestoreOutcome::Restored { .. }))
    }

    /// Emergency path for a corrupt state file: the original settings are unknown, so the only
    /// thing done is to switch OFF the proxy if (and only if) it points at TLC LINK's own local
    /// ports. The corrupt file is archived, never silently deleted.
    pub fn repair_without_snapshot(&self) -> Result<String, String> {
        let ours = [crate::network::system_proxy_server("sing-box"), crate::network::system_proxy_server("xray")];
        let ours_ref: Vec<&str> = ours.iter().map(String::as_str).collect();
        let current = self.backend.read()?;
        let mut note = String::from("هیچ تنظیم پروکسی متعلق به TLC LINK در ویندوز پیدا نشد.");
        if is_ours(&current, &ours_ref) {
            let mut fixed = current.clone();
            fixed.proxy_enable = Some(0);
            self.backend.write(&fixed)?;
            if self.backend.read()? != fixed { return Err("غیرفعال‌سازی پروکسی تأیید نشد".into()); }
            note = "پروکسی محلی TLC LINK غیرفعال شد (مقادیر اصلی قبلی ناشناخته بودند و بازسازی نشدند).".into();
        }
        if self.state_path.exists() {
            let archived = self.state_path.with_extension(format!("json.corrupt-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)));
            fs::rename(&self.state_path, &archived).map_err(|e| format!("بایگانی فایل خراب ممکن نشد: {e}"))?;
        }
        Ok(note)
    }

    /// Independent check that Windows really points at `server` right now.
    pub fn is_applied(&self, server: &str) -> Result<bool, String> {
        let now = self.backend.read()?;
        Ok(now.proxy_enable == Some(1) && now.proxy_server.as_deref() == Some(server))
    }

    fn read_state(&self) -> StateRead {
        let text = match fs::read_to_string(&self.state_path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return StateRead::Missing,
            Err(e) => return StateRead::Corrupt(format!("فایل وضعیت خوانده نشد: {e}")),
        };
        match serde_json::from_str::<ProxyState>(&text) {
            Ok(s) => StateRead::Valid(s),
            Err(e) => StateRead::Corrupt(format!("JSON نامعتبر: {e}")),
        }
    }

    /// Atomic: write temp, fsync, rename over the target.
    fn write_state(&self, state: &ProxyState) -> Result<(), String> {
        let tmp = self.state_path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(state).map_err(|e| e.to_string())?;
        let mut f = fs::File::create(&tmp).map_err(|e| format!("نوشتن فایل وضعیت ممکن نشد: {e}"))?;
        f.write_all(&bytes).map_err(|e| format!("نوشتن فایل وضعیت ممکن نشد: {e}"))?;
        f.sync_all().map_err(|e| format!("همگام‌سازی فایل وضعیت ممکن نشد: {e}"))?;
        drop(f);
        fs::rename(&tmp, &self.state_path).map_err(|e| format!("جایگزینی فایل وضعیت ممکن نشد: {e}"))
    }
}

fn corrupt_message(detail: &str) -> String {
    format!("فایل وضعیت System Proxy خراب است ({detail}). برای ایمنی هیچ تغییری در تنظیمات ویندوز داده نشد. از منوی Tools ← «Emergency Network Reset / بازنشانی اضطراری شبکه» استفاده کنید.")
}

// ───────────────────────────── Windows backend ─────────────────────────────

pub struct WinBackend;

#[cfg(windows)]
impl ProxyBackend for WinBackend {
    fn read(&self) -> Result<ProxySnapshot, String> { win::read_snapshot() }
    fn set_proxy(&self, server: &str, bypass: &str) -> Result<(), String> { win::set_proxy(server, bypass) }
    fn write(&self, s: &ProxySnapshot) -> Result<(), String> { win::write_snapshot(s) }
    fn kill_orphan(&self, pid: u32, image_path: &str) -> Result<bool, String> { crate::winproc::kill_if_image_matches(pid, image_path) }
}

#[cfg(not(windows))]
impl ProxyBackend for WinBackend {
    fn read(&self) -> Result<ProxySnapshot, String> { Err(NOT_WINDOWS.into()) }
    fn set_proxy(&self, _: &str, _: &str) -> Result<(), String> { Err(NOT_WINDOWS.into()) }
    fn write(&self, _: &ProxySnapshot) -> Result<(), String> { Err(NOT_WINDOWS.into()) }
    fn kill_orphan(&self, _: u32, _: &str) -> Result<bool, String> { Ok(false) }
}
#[cfg(not(windows))]
const NOT_WINDOWS: &str = "System Proxy فقط روی ویندوز پشتیبانی می‌شود";

#[cfg(windows)]
mod win {
    use super::ProxySnapshot;
    use windows_sys::Win32::Foundation::ERROR_FILE_NOT_FOUND;
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
        KEY_READ, KEY_SET_VALUE, REG_DWORD, REG_SZ,
    };

    const SUBKEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

    fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }

    struct Key(HKEY);
    impl Drop for Key { fn drop(&mut self) { unsafe { RegCloseKey(self.0); } } }

    fn open(write: bool) -> Result<Key, String> {
        let sub = wide(SUBKEY);
        let mut h: HKEY = core::ptr::null_mut();
        let access = if write { KEY_READ | KEY_SET_VALUE } else { KEY_READ };
        let rc = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, access, &mut h) };
        if rc != 0 { return Err(format!("باز کردن کلید Internet Settings ناموفق بود (کد {rc})")); }
        Ok(Key(h))
    }

    enum Raw { Missing, Sz(String), Dword(u32) }

    fn read_raw(key: &Key, name: &str) -> Result<Raw, String> {
        let n = wide(name);
        let mut ty: u32 = 0;
        let mut len: u32 = 0;
        let rc = unsafe { RegQueryValueExW(key.0, n.as_ptr(), core::ptr::null(), &mut ty, core::ptr::null_mut(), &mut len) };
        if rc == ERROR_FILE_NOT_FOUND { return Ok(Raw::Missing); }
        if rc != 0 { return Err(format!("خواندن {name} ناموفق بود (کد {rc})")); }
        let mut buf = vec![0u8; len as usize];
        let mut len2 = len;
        let rc = unsafe { RegQueryValueExW(key.0, n.as_ptr(), core::ptr::null(), &mut ty, buf.as_mut_ptr(), &mut len2) };
        if rc != 0 { return Err(format!("خواندن {name} ناموفق بود (کد {rc})")); }
        buf.truncate(len2 as usize);
        if ty == REG_DWORD && buf.len() == 4 {
            Ok(Raw::Dword(u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]])))
        } else if ty == REG_SZ {
            let units: Vec<u16> = buf.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
            let end = units.iter().position(|&u| u == 0).unwrap_or(units.len());
            Ok(Raw::Sz(String::from_utf16_lossy(&units[..end])))
        } else {
            // Unknown type: refuse instead of risking a lossy snapshot.
            Err(format!("نوع مقدار رجیستری {name} پشتیبانی نمی‌شود (نوع {ty})"))
        }
    }
    fn read_sz(key: &Key, name: &str) -> Result<Option<String>, String> {
        match read_raw(key, name)? { Raw::Missing => Ok(None), Raw::Sz(s) => Ok(Some(s)), Raw::Dword(_) => Err(format!("{name} باید رشته باشد")) }
    }
    fn read_dword(key: &Key, name: &str) -> Result<Option<u32>, String> {
        match read_raw(key, name)? { Raw::Missing => Ok(None), Raw::Dword(v) => Ok(Some(v)), Raw::Sz(_) => Err(format!("{name} باید DWORD باشد")) }
    }
    fn put_sz(key: &Key, name: &str, value: &str) -> Result<(), String> {
        let n = wide(name);
        let data = wide(value);
        let rc = unsafe { RegSetValueExW(key.0, n.as_ptr(), 0, REG_SZ, data.as_ptr() as *const u8, (data.len() * 2) as u32) };
        if rc != 0 { Err(format!("نوشتن {name} ناموفق بود (کد {rc})")) } else { Ok(()) }
    }
    fn put_dword(key: &Key, name: &str, value: u32) -> Result<(), String> {
        let n = wide(name);
        let data = value.to_le_bytes();
        let rc = unsafe { RegSetValueExW(key.0, n.as_ptr(), 0, REG_DWORD, data.as_ptr(), 4) };
        if rc != 0 { Err(format!("نوشتن {name} ناموفق بود (کد {rc})")) } else { Ok(()) }
    }
    fn delete(key: &Key, name: &str) -> Result<(), String> {
        let n = wide(name);
        let rc = unsafe { RegDeleteValueW(key.0, n.as_ptr()) };
        if rc == 0 || rc == ERROR_FILE_NOT_FOUND { Ok(()) } else { Err(format!("حذف {name} ناموفق بود (کد {rc})")) }
    }

    pub fn read_snapshot() -> Result<ProxySnapshot, String> {
        let k = open(false)?;
        Ok(ProxySnapshot {
            proxy_enable: read_dword(&k, "ProxyEnable")?,
            proxy_server: read_sz(&k, "ProxyServer")?,
            proxy_override: read_sz(&k, "ProxyOverride")?,
            auto_config_url: read_sz(&k, "AutoConfigURL")?,
            auto_detect: read_dword(&k, "AutoDetect")?,
        })
    }

    pub fn set_proxy(server: &str, bypass: &str) -> Result<(), String> {
        let k = open(true)?;
        put_sz(&k, "ProxyServer", server)?;
        put_sz(&k, "ProxyOverride", bypass)?;
        delete(&k, "AutoConfigURL")?; // a PAC script would override the manual proxy
        put_dword(&k, "AutoDetect", 0)?;
        put_dword(&k, "ProxyEnable", 1)?;
        drop(k);
        notify();
        Ok(())
    }

    pub fn write_snapshot(s: &ProxySnapshot) -> Result<(), String> {
        let k = open(true)?;
        // Order: disable first so a half-restored state never leaves a dead proxy enabled.
        put_dword(&k, "ProxyEnable", 0)?;
        match &s.proxy_server { Some(v) => put_sz(&k, "ProxyServer", v)?, None => delete(&k, "ProxyServer")? }
        match &s.proxy_override { Some(v) => put_sz(&k, "ProxyOverride", v)?, None => delete(&k, "ProxyOverride")? }
        match &s.auto_config_url { Some(v) => put_sz(&k, "AutoConfigURL", v)?, None => delete(&k, "AutoConfigURL")? }
        match s.auto_detect { Some(v) => put_dword(&k, "AutoDetect", v)?, None => delete(&k, "AutoDetect")? }
        match s.proxy_enable { Some(v) => put_dword(&k, "ProxyEnable", v)?, None => delete(&k, "ProxyEnable")? }
        drop(k);
        notify();
        Ok(())
    }

    fn notify() {
        #[link(name = "wininet")]
        extern "system" {
            fn InternetSetOptionW(h: *mut core::ffi::c_void, opt: u32, buf: *mut core::ffi::c_void, len: u32) -> i32;
        }
        // 39 = INTERNET_OPTION_SETTINGS_CHANGED, 37 = INTERNET_OPTION_REFRESH
        unsafe {
            InternetSetOptionW(core::ptr::null_mut(), 39, core::ptr::null_mut(), 0);
            InternetSetOptionW(core::ptr::null_mut(), 37, core::ptr::null_mut(), 0);
        }
    }
}

// ───────────────────────────── Tests (in-memory fake backend) ─────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};

    #[derive(Default)]
    struct Fake {
        reg: Mutex<ProxySnapshot>,
        fail_set: AtomicBool,
        fail_write: AtomicBool,
        write_is_lossy: AtomicBool,
        killed: Mutex<Vec<(u32, String)>>,
    }
    struct FakeHandle(Arc<Fake>);
    impl ProxyBackend for FakeHandle {
        fn read(&self) -> Result<ProxySnapshot, String> { Ok(self.0.reg.lock().unwrap().clone()) }
        fn set_proxy(&self, server: &str, bypass: &str) -> Result<(), String> {
            if self.0.fail_set.load(Ordering::SeqCst) { return Err("denied".into()); }
            let mut r = self.0.reg.lock().unwrap();
            r.proxy_server = Some(server.into()); r.proxy_override = Some(bypass.into());
            r.auto_config_url = None; r.auto_detect = Some(0); r.proxy_enable = Some(1);
            Ok(())
        }
        fn write(&self, s: &ProxySnapshot) -> Result<(), String> {
            if self.0.fail_write.load(Ordering::SeqCst) { return Err("denied".into()); }
            let mut r = self.0.reg.lock().unwrap();
            *r = s.clone();
            if self.0.write_is_lossy.load(Ordering::SeqCst) { r.proxy_server = Some("tampered".into()); }
            Ok(())
        }
        fn kill_orphan(&self, pid: u32, path: &str) -> Result<bool, String> { self.0.killed.lock().unwrap().push((pid, path.into())); Ok(true) }
    }

    fn setup(initial: ProxySnapshot) -> (SystemProxy, Arc<Fake>, PathBuf) {
        let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("tlc-proxy-test-{}-{}", std::process::id(), n));
        fs::create_dir_all(&dir).unwrap();
        let fake = Arc::new(Fake::default());
        *fake.reg.lock().unwrap() = initial;
        (SystemProxy::with_backend(&dir, Box::new(FakeHandle(fake.clone()))), fake, dir)
    }
    const OURS: &str = "127.0.0.1:2080";

    #[test]
    fn nothing_to_restore_without_state_file() {
        let (p, _f, d) = setup(ProxySnapshot::default());
        assert!(!p.is_owned());
        assert_eq!(p.restore().unwrap(), RestoreOutcome::NothingToRestore);
        assert_eq!(p.recover_if_dirty().unwrap(), false);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn manual_proxy_is_restored_exactly() {
        let original = ProxySnapshot { proxy_enable: Some(1), proxy_server: Some("corp:8080".into()), proxy_override: Some("*.corp".into()), auto_config_url: None, auto_detect: Some(1) };
        let (p, f, d) = setup(original.clone());
        p.apply(OURS, Some(7), Some(r"C:\x\sing-box.exe")).unwrap();
        assert!(p.is_applied(OURS).unwrap());
        assert!(matches!(p.restore().unwrap(), RestoreOutcome::Restored { snapshot_unreliable: false }));
        assert_eq!(*f.reg.lock().unwrap(), original);
        assert!(!p.is_owned());
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn pac_configuration_is_restored_exactly() {
        let original = ProxySnapshot { proxy_enable: Some(0), proxy_server: None, proxy_override: None, auto_config_url: Some("http://pac.corp/proxy.pac".into()), auto_detect: Some(0) };
        let (p, f, d) = setup(original.clone());
        p.apply(OURS, None, None).unwrap();
        assert_eq!(f.reg.lock().unwrap().auto_config_url, None); // PAC disabled while connected
        p.restore().unwrap();
        assert_eq!(*f.reg.lock().unwrap(), original);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn absent_values_stay_absent_and_are_not_turned_into_zero() {
        let original = ProxySnapshot::default(); // nothing exists in the registry
        let (p, f, d) = setup(original.clone());
        p.apply(OURS, None, None).unwrap();
        p.restore().unwrap();
        let after = f.reg.lock().unwrap().clone();
        assert_eq!(after, original);
        assert_eq!(after.proxy_enable, None);
        assert_ne!(Some(0u32), after.auto_detect);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn second_apply_keeps_the_original_snapshot() {
        let original = ProxySnapshot { proxy_enable: Some(1), proxy_server: Some("corp:8080".into()), ..Default::default() };
        let (p, f, d) = setup(original.clone());
        p.apply(OURS, Some(1), None).unwrap();
        p.apply("127.0.0.1:2081", Some(2), None).unwrap(); // server switch without restore
        p.restore().unwrap();
        assert_eq!(*f.reg.lock().unwrap(), original);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn corrupt_state_blocks_apply_without_touching_registry_and_is_preserved() {
        let original = ProxySnapshot { proxy_enable: Some(1), proxy_server: Some("corp:8080".into()), ..Default::default() };
        let (p, f, d) = setup(original.clone());
        fs::write(d.join(STATE_FILE), "{not json").unwrap();
        assert!(p.is_owned());
        assert!(p.preflight().is_err());
        assert!(p.apply(OURS, None, None).is_err());
        assert!(p.restore().is_err());
        assert!(p.recover_if_dirty().is_err());
        assert_eq!(*f.reg.lock().unwrap(), original);
        assert!(d.join(STATE_FILE).exists(), "corrupt file must not be deleted silently");
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn corrupt_state_emergency_repair_disables_only_our_own_proxy_and_archives_file() {
        let (p, f, d) = setup(ProxySnapshot::default());
        p.apply(OURS, None, None).unwrap();
        fs::write(d.join(STATE_FILE), "garbage").unwrap();
        let note = p.repair_without_snapshot().unwrap();
        assert!(note.contains("غیرفعال"));
        assert_eq!(f.reg.lock().unwrap().proxy_enable, Some(0));
        assert!(!d.join(STATE_FILE).exists());
        let archived = fs::read_dir(&d).unwrap().filter_map(Result::ok).any(|e| e.file_name().to_string_lossy().contains("corrupt"));
        assert!(archived);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn repair_leaves_a_foreign_proxy_alone() {
        let foreign = ProxySnapshot { proxy_enable: Some(1), proxy_server: Some("corp:8080".into()), ..Default::default() };
        let (p, f, d) = setup(foreign.clone());
        fs::write(d.join(STATE_FILE), "garbage").unwrap();
        p.repair_without_snapshot().unwrap();
        assert_eq!(*f.reg.lock().unwrap(), foreign);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn failed_set_proxy_rolls_back_and_removes_state() {
        let original = ProxySnapshot { proxy_enable: Some(0), proxy_server: Some("old:1".into()), ..Default::default() };
        let (p, f, d) = setup(original.clone());
        f.fail_set.store(true, Ordering::SeqCst);
        assert!(p.apply(OURS, None, None).is_err());
        assert_eq!(*f.reg.lock().unwrap(), original);
        assert!(!p.is_owned());
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn failed_restore_keeps_state_for_retry() {
        let (p, f, d) = setup(ProxySnapshot { proxy_enable: Some(0), ..Default::default() });
        p.apply(OURS, None, None).unwrap();
        f.fail_write.store(true, Ordering::SeqCst);
        assert!(p.restore().is_err());
        assert!(p.is_owned());
        f.fail_write.store(false, Ordering::SeqCst);
        assert!(p.restore().is_ok());
        assert!(!p.is_owned());
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn restore_that_does_not_read_back_identically_keeps_the_state_file() {
        let (p, f, d) = setup(ProxySnapshot { proxy_enable: Some(0), ..Default::default() });
        p.apply(OURS, None, None).unwrap();
        f.write_is_lossy.store(true, Ordering::SeqCst);
        assert!(p.restore().is_err());
        assert!(p.is_owned());
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn lost_state_with_our_values_live_is_marked_unreliable() {
        let (p, f, d) = setup(ProxySnapshot { proxy_enable: Some(1), proxy_server: Some(OURS.into()), ..Default::default() });
        p.apply(OURS, None, None).unwrap();
        match p.restore().unwrap() { RestoreOutcome::Restored { snapshot_unreliable } => assert!(snapshot_unreliable), o => panic!("{o:?}") }
        assert_eq!(f.reg.lock().unwrap().proxy_enable, Some(0));
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn recovery_kills_orphan_by_recorded_identity_then_restores() {
        let original = ProxySnapshot { proxy_enable: Some(0), ..Default::default() };
        let (p, f, d) = setup(original.clone());
        p.apply(OURS, Some(4242), Some(r"C:\app\sing-box.exe")).unwrap();
        assert!(p.recover_if_dirty().unwrap());
        assert_eq!(f.killed.lock().unwrap().as_slice(), &[(4242, r"C:\app\sing-box.exe".to_string())]);
        assert_eq!(*f.reg.lock().unwrap(), original);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn state_file_write_is_atomic_json_with_version() {
        let (p, _f, d) = setup(ProxySnapshot::default());
        p.apply(OURS, Some(1), None).unwrap();
        let text = fs::read_to_string(d.join(STATE_FILE)).unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["version"], STATE_VERSION);
        assert!(!d.join("system-proxy-state.json.tmp").exists());
        let _ = fs::remove_dir_all(d);
    }
}
