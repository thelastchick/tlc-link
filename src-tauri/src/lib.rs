mod cores;
pub mod ks;
mod errors;
mod kill_switch;
mod network;
mod probe;
mod protocols;
mod system_proxy;
mod traffic;
mod tun;
mod winproc;

use cores::{CoreKind, CoreManager, CoreStatus};
use kill_switch::KillSwitch;
use serde::Serialize;
use std::{path::PathBuf, sync::{Mutex, MutexGuard, TryLockError}, time::Duration};
use system_proxy::{RestoreOutcome, SystemProxy};
use tauri::{Manager, State};

/// `manager` guards the core process; `op` serialises whole operations (connect / disconnect /
/// reset / mode change) so two commands can never interleave their multi-step transactions.
struct AppState {
    manager: Mutex<CoreManager>,
    op: Mutex<()>,
    startup_notice: Mutex<Option<String>>,
    active_mode: Mutex<Option<network::NetworkMode>>,
    kill_switch: Mutex<KillSwitch>,
    /// When true, the next CloseRequested is allowed to fully quit (tray Exit).
    allow_exit: Mutex<bool>,
    /// When true (default), window close hides to tray instead of quitting.
    close_to_tray: Mutex<bool>,
}

fn lmsg(lang: Option<&str>, fa: &str, ar: &str, en: &str) -> String {
    match lang.unwrap_or("fa").trim().to_ascii_lowercase().as_str() {
        "fa" => fa.to_string(),
        "ar" => ar.to_string(),
        _ => en.to_string(),
    }
}

fn busy_msg(lang: Option<&str>) -> String {
    lmsg(
        lang,
        "عملیات دیگری (اتصال، قطع یا بازنشانی) در حال انجام است؛ چند ثانیه بعد دوباره تلاش کنید.",
        "هناك عملية أخرى قيد التنفيذ (اتصال أو قطع أو إعادة ضبط)؛ حاول مرة أخرى بعد بضع ثوانٍ.",
        "Another operation (connect, disconnect, or reset) is in progress; please try again in a few seconds.",
    )
}

fn unsupported_mode_msg(lang: Option<&str>, detail: &str) -> String {
    lmsg(
        lang,
        &format!("حالت انتخاب‌شده پشتیبانی نمی‌شود: {detail}"),
        &format!("وضع الشبكة المحدد غير مدعوم: {detail}"),
        &format!("Selected network mode is not supported: {detail}"),
    )
}

/// Poison-tolerant lock: a panic in one command must not brick every later command
/// (especially Disconnect / Emergency Reset).
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> { m.lock().unwrap_or_else(|p| p.into_inner()) }

/// Non-blocking acquisition of the operation lock (used by Connect and by status polling).
fn try_op<'a>(
    m: &'a Mutex<()>,
    lang: Option<&str>
) -> Result<MutexGuard<'a, ()>, String> {
    match m.try_lock() {
        Ok(g) => Ok(g),
        Err(TryLockError::Poisoned(p)) => Ok(p.into_inner()),
        Err(TryLockError::WouldBlock) => Err(busy_msg(lang)),
    }
}

#[derive(Serialize)]
struct Capabilities { system_proxy: bool, tun: bool, kill_switch: bool, dns_protection: bool, traffic_stats: bool, subscriptions: bool }

#[derive(Serialize)]
struct RuntimeInfo {
    app_version: &'static str,
    supported_cores: Vec<&'static str>,
    active_core: Option<String>,
    connection: CoreStatus,
    capabilities: Capabilities,
    /// Set once when startup recovery found a dirty previous session (or failed to recover).
    startup_notice: Option<String>,
}

/// Result of `connect_core`. `process_running`, `proxy_applied` and `tunnel_verified`
/// are separate facts on purpose: only `tunnel_verified` means traffic really flowed.
#[derive(Serialize)]
struct ConnectResult {
    #[serde(flatten)]
    status: CoreStatus,
    network_mode: String,
    proxy_applied: bool,
    tunnel_verified: bool,
    latency_ms: Option<u64>,
}

#[derive(Serialize)]
struct ResetStep { name: &'static str, ok: bool, detail: String }
#[derive(Serialize)]
struct ResetReport { ok: bool, steps: Vec<ResetStep> }

/// Extract the primary proxy outbound host/port from a normalized sing-box/xray config
/// and resolve the host to an IP address suitable for WFP permit filters.
///
/// Resolution strategy:
/// 1. If `server` is already an IP literal → use it directly.
/// 2. Otherwise perform a blocking DNS lookup via `ToSocketAddrs`.
/// 3. Prefer the first IPv4 address; fall back to the first IPv6 address.
///
/// Returns `None` only when the outbound cannot be located or DNS fails.
/// Callers that requested Kill Switch must treat `None` as a hard failure (fail-closed).
fn extract_tunnel_endpoint(config_json: &str) -> Option<(String, u16, String, std::net::IpAddr)> {
    let v: serde_json::Value = serde_json::from_str(config_json).ok()?;
    let outbounds = v.get("outbounds")?.as_array()?;
    // Prefer tag "proxy", else first non-direct/block outbound with a server field.
    let candidate = outbounds
        .iter()
        .find(|o| o.get("tag").and_then(|t| t.as_str()) == Some("proxy"))
        .or_else(|| {
            outbounds.iter().find(|o| {
                let t = o.get("type").and_then(|x| x.as_str()).unwrap_or("");
                t != "direct" && t != "block" && t != "dns" && o.get("server").is_some()
            })
        })?;
    let host = candidate.get("server")?.as_str()?.to_string();
    let port = candidate
        .get("server_port")
        .and_then(|p| p.as_u64())
        .or_else(|| candidate.get("port").and_then(|p| p.as_u64()))
        .unwrap_or(443) as u16;
    let proto = candidate
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("tcp")
        .to_string();

    // Fast path: already an IP literal.
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return Some((host, port, proto, ip));
    }

    // Hostname path: resolve via the OS resolver.
    // We intentionally do a blocking lookup here because connect is already a
    // long-running, serialized operation under the `op` lock.
    let addr_str = format!("{host}:{port}");
    let addrs: Vec<std::net::SocketAddr> = match std::net::ToSocketAddrs::to_socket_addrs(&addr_str)
    {
        Ok(iter) => iter.collect(),
        Err(_) => return None,
    };
    if addrs.is_empty() {
        return None;
    }

    // Prefer IPv4 (most common for VPS/edge nodes and simpler WFP filter).
    let ip = addrs
        .iter()
        .find_map(|a| match a.ip() {
            std::net::IpAddr::V4(v4) => Some(std::net::IpAddr::V4(v4)),
            _ => None,
        })
        .or_else(|| addrs.first().map(|a| a.ip()))?;

    Some((host, port, proto, ip))
}

fn app_dirs(app: &tauri::AppHandle) -> Result<(PathBuf, PathBuf), String> {
    let resource = app.path().resource_dir().map_err(|e| e.to_string())?;
    let config = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok((resource, config))
}

/// TUN is supported only through sing-box on Windows. DNS protection is coupled to TUN.
fn check_mode_supported(
    core: &str,
    mode: network::NetworkMode,
    dns_guard: bool,
    lang: Option<&str>,
) -> Result<(), String> {
    if dns_guard && mode != network::NetworkMode::Tun {
        return Err(unsupported_mode_msg(lang, "DNS Leak Guard به System TUN نیاز دارد."));
    }
    match mode {
        network::NetworkMode::SystemProxy => Ok(()),
        network::NetworkMode::Tun if !cfg!(windows) => Err(unsupported_mode_msg(lang, "System TUN فقط روی Windows فعال است.")),
        network::NetworkMode::Tun if !core.eq_ignore_ascii_case("sing-box") && !core.eq_ignore_ascii_case("singbox") => {
            Err(unsupported_mode_msg(lang, "System TUN و DNS Leak Guard در این نسخه فقط با sing-box کار می‌کنند؛ Xray فقط System Proxy دارد."))
        }
        network::NetworkMode::Tun => Ok(()),
    }
}

/// Shared pipeline for validate/connect: detect JSON vs URI, convert, add the network mode & routing mode.
fn prepare_config(
    core: &str,
    config_json: &str,
    network_mode: Option<&str>,
    routing_mode: Option<&str>,
    dns_guard: bool,
    lang: Option<&str>,
) -> Result<(CoreKind, network::NetworkMode, String), String> {
    let kind = CoreKind::parse(core)?;
    let trimmed = config_json.trim();
    let mut config: serde_json::Value = if trimmed.starts_with('{') || trimmed.starts_with('[') {
        serde_json::from_str(trimmed).map_err(|e| {
            format!(
                "{}: {e}",
                lmsg(lang, "JSON نامعتبر است", "تنسيق JSON غير صالح", "Invalid JSON")
            )
        })?
    } else {
        protocols::convert(trimmed, kind.name())?
    };
    if !config.is_object() {
        return Err(lmsg(
            lang,
            "ریشه کانفیگ باید یک JSON object باشد",
            "يجب أن يكون جذر الإعداد كائن JSON",
            "Config root must be a JSON object",
        ));
    }
    let mode = network::NetworkMode::parse(network_mode.unwrap_or("system-proxy"))?;
    network::apply_mode(&mut config, kind.name(), mode, dns_guard)?;
    if let Some(rm) = routing_mode {
        network::apply_routing_mode(&mut config, kind.name(), rm);
    }
    let normalized = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    Ok((kind, mode, normalized))
}

#[tauri::command]
fn runtime_info(lang: Option<String>, state: State<'_, AppState>) -> Result<RuntimeInfo, String> {
    let _ = lang;
    let manager = lock(&state.manager);
    let notice = lock(&state.startup_notice).take();
    Ok(RuntimeInfo {
        app_version: env!("CARGO_PKG_VERSION"),
        supported_cores: vec!["sing-box", "xray"],
        active_core: manager.active_core_name(),
        connection: manager.status(),
        capabilities: Capabilities {
            system_proxy: true,
            tun: cfg!(windows),
            // Keep disabled until the independent service, boot recovery, and Windows
            // packet-capture acceptance matrix are complete. WFP scaffolding alone
            // is not a production Kill Switch capability.
            kill_switch: false,
            dns_protection: cfg!(windows),
            // Live counters via sing-box Clash API (loopback). Xray reports unavailable.
            traffic_stats: true,
            subscriptions: true,
        },
        startup_notice: notice,
    })
}

#[tauri::command(async)]
fn validate_core_config(
    core: String,
    config_json: String,
    network_mode: Option<String>,
    routing_mode: Option<String>,
    dns_guard: Option<bool>,
    lang: Option<String>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let l = lang.as_deref();
    let dns_guard = dns_guard.unwrap_or(false);
    let (kind, mode, normalized) = prepare_config(&core, &config_json, network_mode.as_deref(), routing_mode.as_deref(), dns_guard, l)?;
    check_mode_supported(kind.name(), mode, dns_guard, l)?;
    let (resource_dir, config_dir) = app_dirs(&app)?;
    let mut manager = CoreManager::default();
    manager.set_paths(resource_dir, config_dir);
    manager.validate_json(kind, &normalized).map_err(|e| e.to_string())?;
    Ok(lmsg(l, "اعتبارسنجی هسته موفق بود", "تم التحقق من إعدادات النواة بنجاح", "Core configuration validated successfully"))
}

#[tauri::command(async)]
fn connect_core(
    core: String,
    config_json: String,
    network_mode: Option<String>,
    routing_mode: Option<String>,
    dns_guard: Option<bool>,
    kill_switch: Option<bool>,
    lang: Option<String>,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<ConnectResult, String> {
    let l = lang.as_deref();
    let dns_guard = dns_guard.unwrap_or(false);
    let want_ks = kill_switch.unwrap_or(false);
    if want_ks {
        return Err(lmsg(
            l,
            "Kill Switch در این نسخه غیرفعال است و تا تکمیل آزمون‌های امنیتی ویندوز قابل استفاده نیست.",
            "ميزة Kill Switch معطلة في هذا الإصدار حتى اكتمال اختبارات أمان Windows.",
            "Kill Switch is disabled in this build until the Windows security validation gates are complete.",
        ));
    }
    let (kind, mode, normalized) = prepare_config(&core, &config_json, network_mode.as_deref(), routing_mode.as_deref(), dns_guard, l)?;
    check_mode_supported(kind.name(), mode, dns_guard, l)?;
    if mode == network::NetworkMode::Tun { tun::ensure_elevated()?; }
    let (resource_dir, config_dir) = app_dirs(&app)?;
    let _op = try_op(&state.op, l)?;
    let proxy = SystemProxy::new(&config_dir);

    // Only System Proxy mode touches WinINet proxy settings.
    if mode == network::NetworkMode::SystemProxy { proxy.preflight()?; }
    {
        let mut manager = lock(&state.manager);
        manager.set_paths(resource_dir, config_dir.clone());
        manager.validate_json(kind, &normalized).map_err(|e| e.to_string())?;
    }

    // Step 1 — release the previous session and clean only TLC-owned network state.
    let previous_mode = *lock(&state.active_mode);
    let had_session = lock(&state.manager).is_running() || proxy.is_owned() || previous_mode.is_some();
    if had_session {
        if proxy.is_owned() {
            if let Err(e) = proxy.restore() {
                return Err(format!(
                    "{} {e}",
                    lmsg(l, "تعویض سرور لغو شد؛ پروکسی ویندوز بازیابی نشد و جلسه قبلی دست‌نخورده ماند.", "أُلغي تبديل الخادم؛ لم تتم استعادة وكيل ويندوز وبقيت الجلسة السابقة كما هي.", "Server switch aborted; Windows proxy could not be restored and the previous session was left intact.")
                ));
            }
        }
        if let Err(e) = lock(&state.manager).stop() {
            return Err(format!(
                "{}: {e}",
                lmsg(l, "توقف اتصال قبلی ناموفق بود", "فشل إيقاف الاتصال السابق", "Failed to stop previous connection")
            ));
        }
        if previous_mode == Some(network::NetworkMode::Tun) {
            if let Err(e) = tun::cleanup_owned_interface() {
                *lock(&state.active_mode) = previous_mode;
                return Err(format!("هسته متوقف شد، اما پاکسازی رابط TLC-TUN کامل نشد: {e}. از Tools → Emergency Network Reset استفاده کنید."));
            }
        }
        *lock(&state.active_mode) = None;
    }

    // Step 2 — local ports are needed only by System Proxy mode.
    if mode == network::NetworkMode::SystemProxy {
      for port in network::required_ports(kind.name()) {
        if !probe::port_is_free(port) {
            return Err(lmsg(
                l,
                &format!("پورت محلی {port} توسط برنامه دیگری اشغال است. آن برنامه را ببندید و دوباره تلاش کنید. تنظیمات ویندوز تغییری نکرد."),
                &format!("المنفذ المحلي {port} قيد الاستخدام من قبل تطبيق آخر. أغلقه وحاول مجدداً. لم تتغير إعدادات ويندوز."),
                &format!("Local port {port} is already in use by another application. Close it and try again. Windows settings were not changed."),
            ));
        }
      }
    }

    // Step 3 — start the new core. From here on every failure stops it again.
    let status = match lock(&state.manager).start(kind, &normalized) {
        Ok(status) => status,
        Err(err) => {
            if mode == network::NetworkMode::Tun {
                if let Err(clean_err) = tun::cleanup_owned_interface() {
                    *lock(&state.active_mode) = Some(network::NetworkMode::Tun);
                    return Err(format!("راه‌اندازی هسته ناموفق بود: {err}. پاکسازی رابط TUN نیز ناموفق شد: {clean_err}. از Tools → Emergency Network Reset استفاده کنید."));
                }
            }
            return Err(err.to_string());
        }
    };
    let image = lock(&state.manager).active_image_path();

    // Step 4 — verify the selected path. TUN has no local proxy port; it is verified
    // through the dedicated Windows adapter, installed routes, optional DNS and a TCP smoke test.
    if mode == network::NetworkMode::Tun {
        let ready = tun::wait_until_ready(Duration::from_secs(18), dns_guard);
        if let Err(err) = ready {
            let _ = lock(&state.manager).stop();
            let cleanup = tun::cleanup_owned_interface();
            return Err(match cleanup {
                Ok(()) => err,
                Err(clean_err) => {
                    *lock(&state.active_mode) = Some(network::NetworkMode::Tun);
                    format!("{err} | پاکسازی رابط TUN نیز کامل نشد: {clean_err}. از Tools → Emergency Network Reset استفاده کنید.")
                },
            });
        }
        if let Err(err) = tun::verify_routed_connectivity(Duration::from_secs(7)) {
            let _ = lock(&state.manager).stop();
            let cleanup = tun::cleanup_owned_interface();
            return Err(match cleanup {
                Ok(()) => err,
                Err(clean_err) => {
                    *lock(&state.active_mode) = Some(network::NetworkMode::Tun);
                    format!("{err} | پاکسازی رابط TUN نیز کامل نشد: {clean_err}. از Tools → Emergency Network Reset استفاده کنید.")
                },
            });
        }

        // Kill Switch: arm only when user enabled the toggle, after TUN is verified.
        // Fail-closed policy:
        // - If the user asked for KS and we cannot extract/resolve the server → stop tunnel.
        // - If arming itself fails → stop tunnel + emergency clear.
        // Never leave the user with a "connected" state that lacks the protection they requested.
        let mut ks_note = String::new();
        if cfg!(windows) && want_ks {
            match extract_tunnel_endpoint(&normalized) {
                Some((host, port, proto, ip)) => {
                    let ep = kill_switch::TunnelEndpoint {
                        host: host.clone(),
                        port,
                        protocol: proto,
                    };
                    match lock(&state.kill_switch).begin_prepare(ep, ip) {
                        Ok(()) => {
                            ks_note = lmsg(
                                l,
                                &format!(" | Kill Switch مسلح شد ({} → {}).", host, ip),
                                &format!(" | تم تسليح Kill Switch ({} → {}).", host, ip),
                                &format!(" | Kill Switch armed ({} → {}).", host, ip),
                            );
                        }
                        Err(e) => {
                            // Fail closed: stop tunnel and clean up rather than claim protection.
                            let _ = lock(&state.manager).stop();
                            let _ = tun::cleanup_owned_interface();
                            let _ = lock(&state.kill_switch).emergency_clear();
                            return Err(format!(
                                "{}: {e}",
                                lmsg(
                                    l,
                                    "راه‌اندازی Kill Switch ناموفق بود؛ تونل متوقف و شبکه بازیابی شد",
                                    "فشل تسليح Kill Switch؛ تم إيقاف النفق واستعادة الشبكة",
                                    "Kill Switch arming failed; tunnel stopped and network restored",
                                )
                            ));
                        }
                    }
                }
                None => {
                    // User explicitly requested Kill Switch but we have no usable endpoint.
                    // Fail closed instead of silently running without protection.
                    let _ = lock(&state.manager).stop();
                    let _ = tun::cleanup_owned_interface();
                    let _ = lock(&state.kill_switch).emergency_clear();
                    return Err(lmsg(
                        l,
                        "Kill Switch درخواست شده بود اما آدرس سرور استخراج یا resolve نشد. تونل متوقف شد. از IP مستقیم استفاده کنید یا اتصال DNS را بررسی کنید.",
                        "طُلب Kill Switch لكن تعذر استخراج أو حل عنوان الخادم. تم إيقاف النفق. استخدم IP مباشرًا أو تحقق من اتصال DNS.",
                        "Kill Switch was requested but the server address could not be extracted or resolved. Tunnel stopped. Use a literal IP or check DNS connectivity.",
                    ));
                }
            }
        }

        *lock(&state.active_mode) = Some(network::NetworkMode::Tun);
        let mut result = ConnectResult { status, network_mode: mode.name().into(), proxy_applied: false, tunnel_verified: true, latency_ms: None };
        result.status.detail = format!(
            "{}{}",
            lmsg(
                l,
                if dns_guard { "TUN فعال شد؛ آداپتور، مسیرهای IPv4، DNS روی رابط TUN و اتصال آزمایشی TCP تأیید شدند. برای تأیید نهاییِ نبود نشت DNS، آزمایش مستقل انجام دهید." } else { "TUN فعال شد؛ آداپتور، مسیرهای IPv4 و یک اتصال آزمایشی TCP تأیید شدند." },
                if dns_guard { "تم تفعيل TUN؛ تم التحقق من المحول ومسارات IPv4 وDNS على واجهة TUN واتصال TCP تجريبي. أجرِ اختباراً مستقلاً لتأكيد عدم تسرب DNS." } else { "تم تفعيل TUN؛ تم التحقق من المحول ومسارات IPv4 واتصال TCP تجريبي." },
                if dns_guard { "TUN active: adapter, IPv4 routes, DNS on the TUN interface and a TCP probe were verified. Run an independent DNS leak test for final confirmation." } else { "TUN active: adapter, IPv4 routes and a TCP connectivity probe were verified." },
            ),
            ks_note
        );
        return Ok(result);
    }

    // System Proxy path: prove HTTP and HTTPS through local proxy before changing WinINet.
    let port = network::http_proxy_port(kind.name());
    if !probe::wait_for_port(port, Duration::from_secs(4)) {
        let _ = lock(&state.manager).stop();
        return Err(lmsg(
            l,
            &format!("هسته روی پورت {port} شروع به شنیدن نکرد. تنظیمات ویندوز تغییری نکرد."),
            &format!("لم تبدأ النواة بالاستماع على المنفذ {port}. لم تتغير إعدادات ويندوز."),
            &format!("Core did not start listening on port {port}. Windows settings were not changed."),
        ));
    }
    let check = probe::through_http_proxy(port, Duration::from_secs(8));
    if !lock(&state.manager).refresh_status().process_running {
        let _ = lock(&state.manager).stop();
        return Err(lmsg(l, "هسته هنگام آزمایش اتصال متوقف شد؛ لاگ هسته را بررسی کنید. تنظیمات ویندوز تغییری نکرد.", "توقفت النواة أثناء اختبار الاتصال؛ تحقق من سجل النواة. لم تتغير إعدادات ويندوز.", "Core exited during connectivity probe; check the core log. Windows settings were not changed."));
    }
    if !check.ok {
        let err_tail = check.error.unwrap_or_default();
        let _ = lock(&state.manager).stop();
        return Err(format!("{} {err_tail}", lmsg(l, "اتصال برقرار نشد؛ هسته متوقف شد و تنظیمات ویندوز تغییری نکرد.", "تعذر إنشاء الاتصال؛ تم إيقاف النواة ولم تتغير إعدادات ويندوز.", "Connection probe failed; core stopped and Windows settings were not changed.")));
    }
    let server = network::system_proxy_server(kind.name());
    if let Err(err) = proxy.apply(&server, status.pid, image.as_deref()) {
        let _ = lock(&state.manager).stop();
        return Err(err);
    }
    *lock(&state.active_mode) = Some(network::NetworkMode::SystemProxy);
    let mut result = ConnectResult { status, network_mode: mode.name().into(), proxy_applied: true, tunnel_verified: true, latency_ms: check.latency_ms };
    let ms = check.latency_ms.unwrap_or(0);
    result.status.detail = lmsg(
        l,
        &format!("متصل: HTTP و تونل HTTPS از مسیر پروکسی تأیید شد ({ms} ms) و System Proxy فعال است."),
        &format!("متصل: تم التحقق من نفق HTTP وHTTPS عبر الوكيل ({ms} ms) ووكيل النظام نشط."),
        &format!("Connected: HTTP and HTTPS tunnel verified through proxy ({ms} ms) and System Proxy is active."),
    );
    Ok(result)
}

#[tauri::command(async)]
fn disconnect_core(lang: Option<String>, app: tauri::AppHandle, state: State<'_, AppState>) -> Result<CoreStatus, String> {
    let l = lang.as_deref();
    let (_, config_dir) = app_dirs(&app)?;
    let _op = lock(&state.op);
    let proxy = SystemProxy::new(&config_dir);
    let mode = *lock(&state.active_mode);
    if mode != Some(network::NetworkMode::Tun) {
        if let Err(e) = proxy.restore() {
            return Err(format!(
                "{} {e}",
                lmsg(l, "قطع اتصال انجام نشد: بازیابی پروکسی ویندوز ناموفق بود؛ هسته عمداً فعال ماند.", "فشل قطع الاتصال: تعذرت استعادة وكيل ويندوز؛ بقيت النواة نشطة عمداً.", "Disconnect failed: Windows proxy restore failed; the core was intentionally kept running."),
            ));
        }
    }
    let stopped = lock(&state.manager).stop().map_err(|e| e.to_string())?;
    if mode == Some(network::NetworkMode::Tun) {
        tun::cleanup_owned_interface().map_err(|e| format!("هسته متوقف شد، اما پاکسازی مسیرهای TLC-TUN کامل نشد: {e}. از Tools → Emergency Network Reset استفاده کنید."))?;
    }
    // Explicit Disconnect is the only path that releases Kill Switch protection.
    let _ = lock(&state.kill_switch).begin_release();
    *lock(&state.active_mode) = None;
    Ok(stopped)
}

#[tauri::command(async)]
fn reset_network(lang: Option<String>, app: tauri::AppHandle, state: State<'_, AppState>) -> Result<ResetReport, String> {
    let l = lang.as_deref();
    let (_, config_dir) = app_dirs(&app)?;
    let _op = lock(&state.op);
    let proxy = SystemProxy::new(&config_dir);
    let mut steps: Vec<ResetStep> = Vec::new();

    let core_running = lock(&state.manager).is_running();
    let active_mode = *lock(&state.active_mode);
    let outcome: Result<String, String> = if !proxy.is_owned() {
        Ok(lmsg(
            l,
            "تغییری از طرف TLC LINK در پروکسی ویندوز وجود نداشت.",
            "لم يكن هناك أي تغيير من TLC LINK على وكيل ويندوز.",
            "No Windows proxy changes from TLC LINK were present.",
        ))
    } else {
        let attempt = if core_running { proxy.restore().map(|o| matches!(o, RestoreOutcome::Restored { .. })) } else { proxy.recover_if_dirty() };
        match attempt {
            Ok(_) => Ok(lmsg(
                l,
                "تنظیمات پروکسی ویندوز به حالت قبل از TLC LINK بازگردانده و تأیید شد.",
                "تمت استعادة إعدادات وكيل ويندوز إلى حالتها قبل TLC LINK والتحقق منها.",
                "Windows proxy settings restored to pre-TLC LINK state and verified.",
            )),
            Err(e) if proxy.preflight().is_err() => proxy.repair_without_snapshot().map_err(|e2| format!("{e} | {e2}")),
            Err(e) => Err(e),
        }
    };
    let proxy_ok = outcome.is_ok();
    steps.push(match outcome {
        Ok(detail) => ResetStep { name: "system-proxy", ok: true, detail },
        Err(detail) => ResetStep { name: "system-proxy", ok: false, detail },
    });

    if proxy_ok {
        steps.push(match lock(&state.manager).stop() {
            Ok(_) => ResetStep {
                name: "core",
                ok: true,
                detail: lmsg(l, "هسته فعال متوقف شد (یا هسته‌ای فعال نبود).", "تم إيقاف النواة النشطة.", "Active core stopped (or none was running)."),
            },
            Err(e) => ResetStep {
                name: "core",
                ok: false,
                detail: format!("{}: {e}", lmsg(l, "توقف هسته ناموفق بود", "فشل إيقاف النواة", "Failed to stop core")),
            },
        });
    } else {
        steps.push(ResetStep {
            name: "core",
            ok: !core_running,
            detail: if core_running {
                lmsg(
                    l,
                    "هسته عمداً فعال ماند چون پروکسی ویندوز بازگردانده نشد؛ توقف آن اینترنت شما را قطع می‌کرد. دوباره تلاش کنید.",
                    "بقيت النواة نشطة عمداً لأن استعادة وكيل ويندوز فشلت. حاول مجدداً.",
                    "Core kept running because Windows proxy restore failed; stopping it would break internet access. Retry reset.",
                )
            } else {
                lmsg(l, "هسته‌ای در حال اجرا نبود.", "لم تكن هناك نواة قيد التشغيل.", "No core was running.")
            },
        });
    }

    if proxy_ok && !lock(&state.manager).is_running() {
        let tun_cleanup = tun::cleanup_owned_interface();
        match tun_cleanup {
            Ok(()) => {
                steps.push(ResetStep { name: "tun-routes-dns", ok: true, detail: lmsg(l, "مسیرها و DNS فقط از رابط اختصاصی TLC-TUN پاکسازی شدند.", "تم تنظيف المسارات وDNS من واجهة TLC-TUN المخصصة فقط.", "Routes and DNS were cleaned only from the dedicated TLC-TUN interface.") });
                *lock(&state.active_mode) = None;
            }
            Err(detail) => steps.push(ResetStep { name: "tun-routes-dns", ok: false, detail }),
        }
    } else {
        steps.push(ResetStep { name: "tun-routes-dns", ok: active_mode != Some(network::NetworkMode::Tun), detail: lmsg(l, "پاکسازی TUN انجام نشد چون مرحله توقف هسته کامل نشد.", "لم يتم تنظيف TUN لأن إيقاف النواة لم يكتمل.", "TUN cleanup was skipped because the core stop step did not complete.") });
    }
    // Kill Switch emergency clear — always safe; removes only TLC journal / owned objects.
    {
        let mut ks = lock(&state.kill_switch);
        match ks.emergency_clear() {
            Ok(detail) => steps.push(ResetStep {
                name: "firewall",
                ok: true,
                detail: lmsg(
                    l,
                    &format!("Kill Switch: {detail}"),
                    &format!("Kill Switch: {detail}"),
                    &format!("Kill Switch: {detail}"),
                ),
            }),
            Err(detail) => steps.push(ResetStep {
                name: "firewall",
                ok: false,
                detail: lmsg(
                    l,
                    &format!("پاکسازی Kill Switch ناموفق: {detail}"),
                    &format!("فشل تنظيف Kill Switch: {detail}"),
                    &format!("Kill Switch cleanup failed: {detail}"),
                ),
            }),
        }
    }
    let ok = steps.iter().all(|s| s.ok);
    Ok(ResetReport { ok, steps })
}

#[tauri::command(async)]
fn connection_status(lang: Option<String>, app: tauri::AppHandle, state: State<'_, AppState>) -> Result<CoreStatus, String> {
    let l = lang.as_deref();
    let (_, config_dir) = app_dirs(&app)?;
    let _op = try_op(&state.op, l);
    let _op = match _op {
        Ok(g) => g,
        Err(_) => {
            let mut st = lock(&state.manager).status();
            st.detail = lmsg(l, "عملیات اتصال یا قطع در حال انجام است…", "عملية الاتصال أو قطع الاتصال قيد التنفيذ…", "Connect or disconnect operation in progress…");
            return Ok(st);
        }
    };
    let mut status = lock(&state.manager).refresh_status();
    if !status.process_running {
        let proxy = SystemProxy::new(&config_dir);
        let mode = *lock(&state.active_mode);
        if mode == Some(network::NetworkMode::Tun) {
            // If a previously armed protection journal exists, preserve its block; do not
            // restore normal internet until explicit Disconnect or Emergency Reset.
            let ks_armed = lock(&state.kill_switch).is_armed()
                || matches!(
                    lock(&state.kill_switch).state(),
                    kill_switch::KsState::Armed
                        | kill_switch::KsState::Switching
                        | kill_switch::KsState::BlockedWaitingForTunnel
                );
            if ks_armed {
                let _ = lock(&state.kill_switch).on_unexpected_core_exit();
                status.detail = lmsg(
                    l,
                    "هسته متوقف شد؛ Kill Switch همچنان ترافیک خارج از تونل را مسدود می‌کند. برای بازیابی اینترنت: قطع اتصال یا Emergency Network Reset.",
                    "توقفت النواة؛ Kill Switch ما زال يحظر المرور خارج النفق. لاستعادة الإنترنت: قطع الاتصال أو Emergency Reset.",
                    "Core stopped; Kill Switch still blocks non-tunnel traffic. To restore internet: Disconnect or Emergency Network Reset.",
                );
                // Keep active_mode = Tun so UI shows protected-blocked state.
            } else {
                match tun::cleanup_owned_interface() {
                    Ok(()) => {
                        status.detail = lmsg(l, "هسته متوقف شد؛ مسیرها و DNS رابط TLC-TUN پاکسازی شدند.", "توقفت النواة؛ تمت إزالة مسارات وDNS واجهة TLC-TUN.", "Core stopped; TLC-TUN routes and DNS were cleaned.");
                        *lock(&state.active_mode) = None;
                    }
                    Err(e) => status.detail = format!("هسته متوقف شد، اما پاکسازی TUN ناموفق بود: {e}. از Tools → Emergency Network Reset استفاده کنید."),
                }
            }
        } else if proxy.is_owned() {
            status.detail = match proxy.restore() {
                Ok(_) => lmsg(l, "هسته متوقف شد؛ تنظیمات پروکسی ویندوز بازگردانده شد.", "توقفت النواة؛ تمت استعادة إعدادات وكيل ويندوز.", "Core stopped; Windows proxy settings restored."),
                Err(e) => format!("{}: {e}", lmsg(l, "هسته متوقف شد، اما بازگردانی پروکسی ناموفق بود", "توقفت النواة لكن فشلت استعادة الوكيل", "Core stopped, but proxy restore failed")),
            };
            *lock(&state.active_mode) = None;
        }
    }
    Ok(status)
}

#[tauri::command(async)]
fn measure_server_latency(address: String, timeout_ms: Option<u64>) -> Result<u64, String> {
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(3000).clamp(500, 10000));
    probe::endpoint_tcp_latency(&address, timeout)
}

/// Live upload/download totals from the local sing-box Clash API.
/// Rates are derived on the frontend from successive totals (no fake numbers).
#[tauri::command(async)]
fn traffic_snapshot(state: State<'_, AppState>) -> Result<traffic::TrafficSnapshot, String> {
    let status = lock(&state.manager).status();
    let is_singbox = status
        .core
        .as_deref()
        .map(|c| c.eq_ignore_ascii_case("sing-box") || c.eq_ignore_ascii_case("singbox"))
        .unwrap_or(false);
    Ok(traffic::snapshot(is_singbox, status.process_running))
}

#[tauri::command]
fn convert_profile_uri(core: String, uri: String, lang: Option<String>) -> Result<String, String> {
    let _ = lang;
    let kind = CoreKind::parse(&core)?;
    let converted = protocols::convert(uri.trim(), kind.name())?;
    serde_json::to_string_pretty(&converted).map_err(|e| e.to_string())
}


fn show_main_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn setup_system_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::{
        menu::{Menu, MenuItem},
        tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
        Manager,
    };

    let show_i = MenuItem::with_id(app, "tray-show", "Show TLC LINK", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "tray-quit", "Exit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no default window icon for tray"))?;

    let _tray = TrayIconBuilder::with_id("tlc-main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("TLC LINK")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "tray-show" => show_main_window(app),
            "tray-quit" => {
                if let Some(state) = app.try_state::<AppState>() {
                    *lock(&state.allow_exit) = true;
                }
                // Exit triggers RunEvent::Exit which already restores proxy / TUN / KS.
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    let _ = _tray;
    Ok(())
}


#[tauri::command]
fn set_close_to_tray(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    *lock(&state.close_to_tray) = enabled;
    Ok(())
}

pub fn run() {
    // KillSwitch is opened with a temporary path; real config dir is applied in setup.
    let placeholder_ks = KillSwitch::open(std::path::Path::new("."));
    tauri::Builder::default()
        .manage(AppState {
            manager: Mutex::new(CoreManager::default()),
            op: Mutex::new(()),
            startup_notice: Mutex::new(None),
            active_mode: Mutex::new(None),
            kill_switch: Mutex::new(placeholder_ks),
            allow_exit: Mutex::new(false),
            close_to_tray: Mutex::new(true),
        })
        .setup(|app| {
            if let Ok(dir) = app.path().app_config_dir() {
                // Replace placeholder with journal under the real config directory.
                *lock(&app.state::<AppState>().kill_switch) = KillSwitch::open(&dir);
                let mut notices = Vec::new();
                match SystemProxy::new(&dir).recover_if_dirty() {
                    Ok(true) => notices.push("Previous TLC LINK session did not exit cleanly; Windows proxy settings were restored.".to_string()),
                    Ok(false) => {},
                    Err(e) => notices.push(format!("Automatic System Proxy recovery failed: {e}")),
                }
                // TUN cleanup is deliberately scoped to the TLC-owned adapter name.
                if cfg!(windows) && tun::owned_interface_present() {
                    if tun::ensure_elevated().is_ok() {
                        if let Err(e) = tun::cleanup_owned_interface() { notices.push(format!("TLC-TUN startup cleanup needs attention: {e}")); }
                    } else {
                        notices.push("A TLC-TUN adapter from a previous session was found. Reopen TLC LINK with Run as administrator and use Tools → Emergency Network Reset before reconnecting.".to_string());
                    }
                }
                // Kill Switch: reconcile journal. If we were Armed/Blocked, re-apply hard block
                // (fail closed) or clear on failure so the user is never left half-protected.
                // Also runs when the on-disk journal was corrupt (orphan WFP sweep).
                {
                    let state = app.state::<AppState>();
                    let mut ks = lock(&state.kill_switch);
                    let snap = ks.status_snapshot();
                    let needs_reconcile = ks.journal_was_corrupt()
                        || (snap.journal_present
                            && matches!(
                                snap.state,
                                kill_switch::KsState::Reconciling
                                    | kill_switch::KsState::BlockedWaitingForTunnel
                                    | kill_switch::KsState::Armed
                                    | kill_switch::KsState::Switching
                                    | kill_switch::KsState::Preparing
                            ));
                    if needs_reconcile {
                        match ks.reconcile_on_startup() {
                            Ok(msg) if !msg.is_empty() => notices.push(msg),
                            Ok(_) => {}
                            Err(e) => notices.push(format!("Kill Switch recovery: {e}")),
                        }
                    }
                }
                *lock(&app.state::<AppState>().startup_notice) = if notices.is_empty() { None } else { Some(notices.join(" | ")) };
            }

            // System tray: always present so the VPN can keep running with the window hidden.
            if let Err(e) = setup_system_tray(app) {
                eprintln!("TLC LINK tray setup failed: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            runtime_info,
            validate_core_config,
            connect_core,
            disconnect_core,
            connection_status,
            convert_profile_uri,
            reset_network,
            measure_server_latency,
            traffic_snapshot,
            set_close_to_tray
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Default: hide to tray so the core / Kill Switch keep running.
                // Full quit only when allow_exit was set (tray → Exit).
                let (allow, to_tray) = window
                    .app_handle()
                    .try_state::<AppState>()
                    .map(|s| (*lock(&s.allow_exit), *lock(&s.close_to_tray)))
                    .unwrap_or((false, true));
                if !allow && to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("TLC LINK failed to start")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Some(state) = app.try_state::<AppState>() {
                    let mode = *lock(&state.active_mode);
                    if let Some(dir) = app.path().app_config_dir().ok() {
                        if mode != Some(network::NetworkMode::Tun) { let _ = SystemProxy::new(&dir).restore(); }
                    }
                    let _ = lock(&state.manager).stop();
                    if mode == Some(network::NetworkMode::Tun) { let _ = tun::cleanup_owned_interface(); }
                    // Explicit process exit: request release of any Kill Switch state.
                    let _ = lock(&state.kill_switch).begin_release();
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broken_json_reports_a_json_error_not_a_uri_error() {
        let err = prepare_config("sing-box", "{\"a\":", None, None, false, Some("en")).unwrap_err();
        assert!(err.contains("JSON"));
    }

    #[test]
    fn uri_is_converted_and_gets_the_local_inbound() {
        let (_, mode, cfg) = prepare_config("sing-box", "trojan://p@host.test:443?sni=host.test", Some("system-proxy"), Some("rule"), false, Some("en")).unwrap();
        assert_eq!(mode, network::NetworkMode::SystemProxy);
        assert!(cfg.contains("tlc-inbound"));
    }

    #[test]
    fn direct_routing_mode_sets_final_direct() {
        let (_, _, cfg) = prepare_config("sing-box", "trojan://p@host.test:443?sni=host.test", Some("system-proxy"), Some("direct"), false, Some("en")).unwrap();
        assert!(cfg.contains("\"final\":\"direct\""));
    }

    #[test]
    fn array_root_is_rejected() {
        assert!(prepare_config("xray", "[]", None, None, false, Some("en")).is_err());
    }

    #[test]
    fn tun_requires_sing_box_and_dns_guard_requires_tun() {
        assert!(check_mode_supported("xray", network::NetworkMode::Tun, false, Some("en")).is_err());
        assert!(check_mode_supported("sing-box", network::NetworkMode::Tun, true, Some("en")).is_ok() || !cfg!(windows));
        assert!(check_mode_supported("sing-box", network::NetworkMode::SystemProxy, true, Some("en")).is_err());
        assert!(check_mode_supported("sing-box", network::NetworkMode::SystemProxy, false, Some("en")).is_ok());
    }

    #[test]
    fn operation_lock_rejects_overlap_but_survives_poisoning() {
        let m = Mutex::new(());
        let first = try_op(&m, Some("en")).unwrap();
        assert_eq!(try_op(&m, Some("en")).unwrap_err(), busy_msg(Some("en")));
        drop(first);
        assert!(try_op(&m, Some("en")).is_ok());
        let p = std::sync::Arc::new(Mutex::new(()));
        let p2 = p.clone();
        let _ = std::thread::spawn(move || { let _g = p2.lock().unwrap(); panic!("poison"); }).join();
        assert!(try_op(&p, Some("en")).is_ok());
    }

    #[test]
    fn extract_endpoint_accepts_ipv4_literal() {
        let cfg = r#"{"outbounds":[{"type":"trojan","tag":"proxy","server":"203.0.113.10","server_port":443}]}"#;
        let (host, port, proto, ip) = extract_tunnel_endpoint(cfg).expect("literal IPv4");
        assert_eq!(host, "203.0.113.10");
        assert_eq!(port, 443);
        assert_eq!(proto, "trojan");
        assert_eq!(ip, "203.0.113.10".parse::<std::net::IpAddr>().unwrap());
    }

    #[test]
    fn extract_endpoint_accepts_ipv6_literal() {
        let cfg = r#"{"outbounds":[{"type":"vless","tag":"proxy","server":"2001:db8::1","server_port":8443}]}"#;
        let (host, port, _, ip) = extract_tunnel_endpoint(cfg).expect("literal IPv6");
        assert_eq!(host, "2001:db8::1");
        assert_eq!(port, 8443);
        assert_eq!(ip, "2001:db8::1".parse::<std::net::IpAddr>().unwrap());
    }

    #[test]
    fn extract_endpoint_prefers_proxy_tag() {
        let cfg = r#"{"outbounds":[
            {"type":"direct","tag":"direct"},
            {"type":"trojan","tag":"proxy","server":"198.51.100.5","server_port":443},
            {"type":"trojan","tag":"backup","server":"203.0.113.9","server_port":443}
        ]}"#;
        let (host, _, _, _) = extract_tunnel_endpoint(cfg).expect("proxy tag");
        assert_eq!(host, "198.51.100.5");
    }

    #[test]
    fn extract_endpoint_returns_none_for_missing_server() {
        let cfg = r#"{"outbounds":[{"type":"direct","tag":"direct"}]}"#;
        assert!(extract_tunnel_endpoint(cfg).is_none());
    }
}
