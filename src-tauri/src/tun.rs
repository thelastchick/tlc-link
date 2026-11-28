//! Windows TUN checks and narrowly-scoped cleanup for the TLC-owned adapter.
//! This module never changes the default firewall policy.

use std::{net::{SocketAddr, TcpStream}, thread, time::{Duration, Instant}};

#[cfg(windows)]
#[link(name = "shell32")]
extern "system" { fn IsUserAnAdmin() -> i32; }

#[cfg(windows)]
fn run_powershell(script: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
        .creation_flags(0x0800_0000)
        .output()
        .map_err(|e| format!("اجرای بررسی شبکه ویندوز ناموفق بود: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        Err(if detail.is_empty() { "فرمان شبکه ویندوز با خطا پایان یافت".into() } else { detail })
    }
}

#[cfg(windows)]
pub fn ensure_elevated() -> Result<(), String> {
    let elevated = unsafe { IsUserAnAdmin() } != 0;
    if elevated { Ok(()) } else {
        Err("برای System TUN باید TLC-LINK.exe را با گزینه Run as administrator اجرا کنید. برنامه را ببندید و دوباره با دسترسی مدیر باز کنید.".into())
    }
}

#[cfg(not(windows))]
pub fn ensure_elevated() -> Result<(), String> {
    Err("System TUN در این نسخه فقط روی Windows پشتیبانی می‌شود.".into())
}

/// Wait for the named Wintun adapter and its auto-routes. The alias is fixed by TLC LINK.
#[cfg(windows)]
pub fn wait_until_ready(timeout: Duration, require_dns: bool) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    let script = if require_dns {
        "$ErrorActionPreference='Stop'; $a=Get-NetAdapter -Name 'TLC-TUN' -ErrorAction SilentlyContinue; if(-not $a -or $a.Status -ne 'Up'){exit 2}; $r=@(Get-NetRoute -InterfaceIndex $a.ifIndex -ErrorAction SilentlyContinue); $v4=@($r | Where-Object {$_.DestinationPrefix -in @('0.0.0.0/1','128.0.0.0/1')}); $d=@(Get-DnsClientServerAddress -InterfaceIndex $a.ifIndex -ErrorAction SilentlyContinue | ForEach-Object {$_.ServerAddresses} | Where-Object {$_}); if($v4.Count -lt 2 -or -not ($d -contains '172.19.0.2')){exit 3}; exit 0"
    } else {
        "$ErrorActionPreference='Stop'; $a=Get-NetAdapter -Name 'TLC-TUN' -ErrorAction SilentlyContinue; if(-not $a -or $a.Status -ne 'Up'){exit 2}; $r=@(Get-NetRoute -InterfaceIndex $a.ifIndex -ErrorAction SilentlyContinue); $v4=@($r | Where-Object {$_.DestinationPrefix -in @('0.0.0.0/1','128.0.0.0/1')}); if($v4.Count -lt 2){exit 3}; exit 0"
    };
    while Instant::now() < deadline {
        if run_powershell(script).is_ok() { return Ok(()); }
        thread::sleep(Duration::from_millis(400));
    }
    Err(if require_dns {
        "آداپتور TUN یا مسیرها یا DNS آن در ویندوز آماده نشدند؛ هسته متوقف و تنظیمات TLC-TUN بازیابی خواهند شد.".into()
    } else {
        "آداپتور TUN یا مسیرهای ویندوز در زمان مجاز آماده نشدند؛ هسته متوقف و مسیرهای TLC-TUN بازیابی خواهند شد.".into()
    })
}

#[cfg(not(windows))]
pub fn wait_until_ready(_: Duration, _: bool) -> Result<(), String> {
    Err("بررسی TUN فقط روی Windows انجام می‌شود.".into())
}

/// Read-only check used at startup to detect a leftover TLC-owned adapter.
#[cfg(windows)]
pub fn owned_interface_present() -> bool {
    run_powershell("$a=Get-NetAdapter -Name 'TLC-TUN' -ErrorAction SilentlyContinue; if($a){exit 0}else{exit 1}").is_ok()
}

#[cfg(not(windows))]
pub fn owned_interface_present() -> bool { false }

/// Remove routes and DNS settings only from the dedicated TLC-TUN interface.
/// It does not touch user routes, the system proxy, or firewall policy.
#[cfg(windows)]
pub fn cleanup_owned_interface() -> Result<(), String> {
    // Only touch the dedicated TLC-TUN interface. Fail loudly if Windows rejects
    // route/DNS cleanup so the UI can show Emergency Network Reset guidance.
    let script = "$ErrorActionPreference='Stop'; $a=Get-NetAdapter -Name 'TLC-TUN' -ErrorAction SilentlyContinue; if($a){ $routes=@(Get-NetRoute -InterfaceIndex $a.ifIndex -ErrorAction SilentlyContinue); if($routes.Count -gt 0){ $routes | Remove-NetRoute -Confirm:$false -ErrorAction Stop }; Set-DnsClientServerAddress -InterfaceIndex $a.ifIndex -ResetServerAddresses -ErrorAction Stop }; exit 0";
    run_powershell(script)
}

#[cfg(not(windows))]
pub fn cleanup_owned_interface() -> Result<(), String> { Ok(()) }

/// Confirm at least one outbound TCP connection succeeds after TUN routes are installed.
/// This is a connectivity smoke test, not a throughput benchmark or a DNS leak audit.
pub fn verify_routed_connectivity(timeout: Duration) -> Result<(), String> {
    // Multiple literal-IP targets reduce false negatives on networks that block
    // one public resolver. The total probe time never exceeds `timeout`.
    // This only verifies basic TCP reachability, not speed or DNS leak resistance.
    let targets: [SocketAddr; 3] = [
        "1.1.1.1:443".parse().expect("literal socket address"),
        "8.8.8.8:443".parse().expect("literal socket address"),
        "9.9.9.9:443".parse().expect("literal socket address"),
    ];
    let deadline = Instant::now() + timeout;
    let mut errors = Vec::new();
    for target in targets {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() { break; }
        match TcpStream::connect_timeout(&target, remaining.min(Duration::from_secs(3))) {
            Ok(stream) => { drop(stream); return Ok(()); }
            Err(err) => errors.push(format!("{target}: {err}")),
        }
    }
    Err(format!("هیچ‌یک از مقصدهای آزمایشی از مسیر TUN پاسخ ندادند: {}", errors.join(" | ")))
}

#[cfg(test)]
mod tests {
    #[test]
    fn interface_alias_is_fixed_and_scoped() {
        // The cleanup script must only reference the TLC-owned interface.
        let source = include_str!("tun.rs");
        assert!(source.contains("Get-NetAdapter -Name 'TLC-TUN'"));
        assert!(source.contains("Get-NetRoute -InterfaceIndex $a.ifIndex"));
        assert!(source.contains("0.0.0.0/1"));
        assert!(source.contains("172.19.0.2"));
        // Build forbidden command names from fragments so this test does not
        // match its own assertion text when scanning the complete source file.
        let firewall_profile_command = ["Set-NetFirewall", "Profile"].concat();
        let firewall_reset_command = ["netsh advfirewall", "reset"].concat();
        assert!(!source.contains(&firewall_profile_command));
        assert!(!source.contains(&firewall_reset_command));
    }
}
