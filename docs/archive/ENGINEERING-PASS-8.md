# TLC LINK — Engineering Pass 8

Status: **engineering preview. Not compiled and not run on Windows in the authoring environment**
(no Rust toolchain, no Windows, no network). Run `cargo test` first, then the Windows checklist below.

## Delivered in source

**Windows System Proxy (roadmap stage 1 + crash recovery from stage 5)** — `system_proxy.rs`
- Snapshot of ProxyEnable / ProxyServer / ProxyOverride / AutoConfigURL / AutoDetect is written to
  `system-proxy-state.json` BEFORE any change. The file exists exactly while TLC LINK owns the settings.
- Apply -> independent read-back verification -> on any failure the snapshot is restored and the core stopped.
- Restore on: Disconnect, normal exit (`RunEvent::Exit`), core crash (status watchdog), and next startup
  when the state file is found (`recover_if_dirty`, which also kills an orphaned core of the recorded PID
  only if the image name still matches).
- WinINet is notified with `InternetSetOptionW` (SETTINGS_CHANGED + REFRESH).
- New command `reset_network` = emergency switch (stop core + restore proxy).
- Xray now also gets an HTTP inbound (port 2081) because WinINet expects an HTTP proxy; sing-box uses `mixed` (2080).

**Real connectivity check (roadmap stage 2, partly 3)** — `probe.rs`
- After the core starts, one HTTP request to `generate_204` is sent THROUGH the local proxy.
- Windows settings are touched only if that probe returns 204. If it fails, the core is stopped and
  nothing changes in Windows. The UI shows "Connected" only when `tunnel_verified` is true.
- The measured round trip is returned as `latency_ms` (a real number for the selected server).

**Protocol conversion fixes** — `protocols.rs`
- VMess links whose Base64 contains `/` no longer break.
- `+` in passwords/UUIDs is preserved (it was turned into a space); percent-decoder can no longer panic on UTF-8.
- VLESS/Trojan: ws, grpc, httpupgrade, http transports; `fp` (uTLS fingerprint; Reality defaults to chrome) and `alpn`.
- Unsupported parameters now return an error instead of being dropped silently
  (VLESS encryption, TCP headerType, xhttp/kcp/quic transports, VMess http-obfuscation).
- sing-box default log level is `warn` (no visited domains written to disk).
- New unit tests for all of the above.

**Process / UI**
- Tauri commands run off the main thread (`#[tauri::command(async)]`) so the UI no longer freezes on connect.
- Core console windows are hidden (`CREATE_NO_WINDOW`).
- Development binary path no longer compiled into release builds.
- Broken JSON now reports a JSON error instead of "URI must contain ://".
- QR libraries are loaded from `frontend/vendor/` (run `scripts/fetch-vendor.ps1` once; it pins SHA-256).
  CSP no longer allows remote scripts.
- Both build scripts computed the project root wrongly (pointed at `scripts/`); fixed.

## Still NOT done (do not advertise)
- TUN mode: config generation only; privileges, routes, DNS, IPv6, recovery untested.
- Kill Switch / DNS Guard: UI preferences only.
- Real traffic counters and duration panel (needs sing-box `clash_api` / Xray stats API).
- Subscriptions, TUIC, WireGuard, Hysteria2 for Xray, VMess `h2`.
- Secure credential storage (profiles remain in WebView localStorage; backups are plaintext).
- Job Object so the core dies with the app on a hard kill (startup recovery covers this after the next launch).
- The UI still loads Google Fonts remotely; the single inline script still requires `'unsafe-inline'`.

## Windows acceptance tests for this pass
1. Connect -> `reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings"` shows 127.0.0.1:2080 (sing-box) / :2081 (Xray).
2. Disconnect -> values identical to before (try with a pre-existing manual proxy and with a PAC URL).
3. Kill `sing-box.exe` in Task Manager -> within ~3 s the UI shows disconnected and the proxy is restored.
4. Kill `TLC LINK.exe` -> start it again -> proxy restored, orphan core gone.
5. Dead/blocked server -> Connect fails with a reason, Windows settings unchanged.
6. Occupy port 2080 -> clear error, nothing changed.
