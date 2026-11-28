# TLC LINK — Engineering Pass 9

**Status: source changes applied, NOT compiled, NOT run.** The authoring environment had no Rust
toolchain, no Windows and no network. Everything below is source-level work that must pass
`cargo test` and the Windows checklist (`RELEASE-CHECKLIST.md`) before any claim of readiness.
This package is therefore **not** a verified release candidate, whatever the archive is called.

## Completed in source (unverified at runtime)

### Build blockers fixed
- `Cargo.toml` had no `[lib]` target although `main.rs` calls `tlc_link_lib::run()` — the crate could not link. Added.
- `tauri.conf.json` referenced no icons; Windows bundling needs `icon.ico`. Icons generated from the existing brand chick (`src-tauri/icons/`).

### System Proxy (`system_proxy.rs`) — rewritten around a `ProxyBackend` trait
- Snapshot of ProxyEnable / ProxyServer / ProxyOverride / AutoConfigURL / AutoDetect; "absent" (`None`) is distinct from `0` / `""`; absent values are *deleted* on restore, not written as zero.
- Registry access now uses the Win32 registry API (`windows-sys`) instead of parsing `reg.exe` output.
- State file: versioned, atomic (temp + fsync + rename). A corrupt file is detected as `Corrupt`, never treated as missing: no registry change is made on top of it, the file is preserved, and the user is pointed to Emergency Network Reset. Reset archives it (`*.corrupt-<ts>`) and only disables a proxy that points at TLC LINK's own ports.
- After apply: registry re-read and verified, rollback on mismatch. After restore: registry re-read and compared with the snapshot; the state file is deleted only if equal; otherwise it is kept for retry.
- If the state file was lost while TLC LINK's own values were live, the snapshot is marked `snapshot_unreliable` (falls back to "proxy disabled") instead of capturing our own values as the "original".
- 14 unit tests with an in-memory fake backend: manual proxy, PAC, absent values, double-apply keeps original, corrupt state, failed set/restore, lossy restore, orphan kill by identity, atomic write.

### Transactional connect / switch / reset (`lib.rs`)
- `op` mutex serialises Connect / Disconnect / Reset; a second Connect is rejected (BUSY), Disconnect/Reset wait, status polling never blocks or races.
- Connect order: preflight (state file sane, config validated by the real core) → release previous session **Windows first, then core** → ports free → start → port listening → real probe → core still alive → apply + verify. Every failure stops the new core; Windows is untouched until the probe passes.
- Disconnect and app exit restore Windows **before** stopping the core; if restore fails the core is deliberately left running (Windows must never point at a dead port) and the user is told to use Emergency Reset.
- Server switch restores the original snapshot first, so the saved snapshot is never TLC LINK's own value.
- Port-ownership bug fixed: previously a foreign process already listening on 2080 would have satisfied the probe. All required ports (2080; Xray also 2081) must now be free before the core starts.
- `reset_network` returns a per-step report (`system-proxy`, `core`, `dns`, `route`, `firewall`); dns/route/firewall honestly report "not managed by this build".
- Mutexes are poison-tolerant so one panicking command cannot disable Disconnect / Reset.
- Startup recovery result is surfaced to the UI (`runtime_info.startup_notice`).

### Probe (`probe.rs`)
Real HTTP 204 through the local proxy **plus** an HTTPS `CONNECT` tunnel check. Failures classified: port closed, timeout, connection reset, upstream 5xx, bad response. No TLS handshake is performed (no TLS crate); CONNECT proves the tunnel reaches the host on :443.

### Process safety (`winproc.rs`, `cores/process.rs`)
- Core is assigned to a kill-on-close **Job Object**: a hard kill of TLC LINK terminates the core. (`CoreStatus.orphan_protection` reports whether it was attached.)
- Orphan cleanup compares the **full executable path** (`QueryFullProcessImageNameW`) before `TerminateProcess` — never PID alone. Startup recovery remains as backup.

### Protocols (`protocols.rs`)
Unknown / unsupported query parameters are now **rejected with their names** (values are not echoed) instead of silently dropped (e.g. Hysteria2 `obfs`, Shadowsocks `plugin`); unknown `security` values rejected; Trojan `peer` alias supported; gRPC `mode` limited to gun/auto. New tests incl. special characters in passwords, IPv6, Reality links with `spx`.

### Frontend / security
- Inline `<script>`/`<style>` moved to `app.js` / `app.css`; CSP is now `script-src 'self'; style-src 'self'; style-src-attr 'unsafe-inline'` (the 15 inline `style=` attributes still need the attribute-only exception). No remote origins in CSP.
- Google Fonts removed (system font fallbacks: Segoe UI / Tahoma / Consolas). Bundling Vazirmatn/Inter/JetBrains Mono locally is still open.
- Kill Switch, DNS Protection and TUN toggles are disabled and labelled **Coming Soon** in all languages; saved "on" values are forced off at startup.
- Menu *Tools → Emergency Network Reset* with per-step log output.
- Stale message "system proxy auto-configuration is not yet enabled" replaced.
- Log redaction (`redactSecrets`): share links, token/sub URLs, UUIDs, password/pbk/private-key values.
- Failed Disconnect keeps the UI in the connected state (the core is still running).
- `tests/frontend-checks.mjs` (Node only): duplicate IDs, no remote origins, strict CSP, inert toggles, log redaction — **passes**.

## Explicitly NOT done (do not advertise)
| Area | State |
|---|---|
| **TUN** | Gated off in the backend (`connect_core` refuses it) and disabled in the UI. No adapter lifecycle, route/default-route, DNS, IPv6, loop prevention, rollback or traffic verification exists. Xray has no TUN path designed. |
| **Kill Switch** (Windows Firewall) | Not implemented. |
| **DNS Protection / leak test** | Not implemented. |
| **Traffic statistics** (Clash API / Xray stats) | Not implemented; UI must show "Not Available". |
| **Real per-server Ping** | Not implemented (Ping All still reports unavailable). |
| **Subscriptions** (fetch, base64, scheduled update, dedupe, keep-on-failure) | Not implemented in the backend. |
| **TUIC, WireGuard, Hysteria2-for-Xray, VMess h2** | Rejected with "Unsupported protocol" (not guessed). |
| **Secure credential storage** (DPAPI / Credential Manager) | Profiles remain in WebView `localStorage`; backups remain plaintext. |
| **QR vendor libraries** | `frontend/vendor/` is empty in this package; run `scripts/fetch-vendor.ps1` (needs internet). QR works offline only after that. |
| **Core binaries** | Not included. Official sing-box and Xray releases must be downloaded, version-pinned and hashed (`scripts/verify-cores.ps1`). |
| Build / installer / portable | Never executed. |

## Known risks to check first when a toolchain is available
1. `windows-sys` 0.59 signatures (HKEY/HANDLE as raw pointers, `QueryFullProcessImageNameW`, `SetInformationJobObject`) were written from memory — expect possible compile fixes in `winproc.rs` / `system_proxy.rs::win`.
2. `AssignProcessToJobObject` can fail if the app already runs inside a non-breakaway job; the failure is reported via `orphan_protection=false`, not hidden.
3. A switch between servers has a short direct-connection window (no Kill Switch).
4. A crashed core is noticed by the 2.5 s status poll; Windows may point at a dead port for up to ~2.5 s until restore.
5. Connect holds the operation lock for up to ~25 s in the worst case (0.9 s start + 4 s listen + 2×8 s probe + validation).

## Core versions
Not pinned yet — no binaries were obtained. Record the exact official sing-box and Xray versions and SHA-256 in `THIRD-PARTY-NOTICES.md` / `src-tauri/binaries/cores.lock.json` when they are downloaded.
