# TLC LINK — The Last Chick (Desktop, Engineering Pass 9)

> **Current packaging target:** `0.2.0-rc.1` (Windows Portable RC). See `docs/RC-PREPARATION.md` and `docs/KNOWN-LIMITATIONS.md`.

> **STATUS: engineering source drop — not compiled, not run on Windows, not a verified release.**
> Read `docs/ENGINEERING-PASS-9.md` (what changed, what is NOT done) and `docs/RELEASE-CHECKLIST.md` (real test results; most are NOT TESTED).
> Working in source: transactional **System Proxy** mode (snapshot/restore, real traffic probe, crash recovery, Job Object, Emergency Network Reset).
> **Experimental / gated:** Windows TUN and DNS Leak Guard are present as an unverified sing-box-only path. Kill Switch remains disabled pending its Windows service/recovery/packet-capture release gates. Live traffic statistics are available only for sing-box; Xray statistics are unavailable. These are disabled in the UI ("Coming Soon").

## Original foundation notes (Pass 1)

This is the first implementation increment of TLC LINK based on the supplied `TLC-LINK.html` design. The original visual frontend is preserved in `frontend/index.html`; only demo endpoints were removed, a core selector was added, and the fake connect/disconnect toggle was replaced with a native Tauri command call.

## What is implemented in this increment

- Tauri 2 + Rust desktop application structure for Windows.
- Two-core selection model: `sing-box` and `xray`.
- Native process manager with start, stop, and process-status operations.
- JSON config validation delegated to the selected core executable.
- No fake server profiles or demo credentials are preloaded.
- The connect button no longer pretends a URI profile is connected. Until the URI-to-core adapters are completed, it accepts a raw JSON configuration for the selected core and reports unsupported input clearly.
- Installer targets (NSIS and MSI) are configured; portable packaging is a separate release task.

## Important limitations — not production VPN yet

This is a real desktop/native foundation, not a finished VPN client. The following are deliberately not claimed as implemented:

1. URI/subscription conversion adapters for VLESS, VMess, Trojan, Shadowsocks, Hysteria2, TUIC, WireGuard, SOCKS and other formats.
2. Xray JSON vs sing-box JSON schema detection and conversion.
3. Windows TUN adapter setup, route management, System Proxy, DNS integration, firewall/Kill Switch and guaranteed network recovery.
4. Real latency probes, QR decoding/encoding, subscription scheduling, secure credential storage and updater.
5. End-to-end tunnel verification. A started core process is not proof that traffic is successfully routed.

The frontend explicitly reports that a running core process still requires a separate network/tunnel check. Do not use this build as a production VPN until the remaining items are implemented and tested.

## Requirements

- Windows 10/11 x64
- Node.js LTS and npm
- Rust stable toolchain with the MSVC target
- Microsoft C++ Build Tools / Windows SDK
- WebView2 Runtime

## Add core binaries

Obtain official Windows x64 releases from the respective upstream projects and place the executables here:

- `src-tauri/binaries/sing-box/sing-box.exe`
- `src-tauri/binaries/xray/xray.exe`

Do not rename unrelated executables to these names. Check the upstream release signatures/checksums and licenses. The Tauri resource configuration must include these paths before producing a distributable package, and `CoreManager` expects them under the packaged `binaries/<core>/` resource directory.

## One-time: local QR libraries

Run `scripts/fetch-vendor.ps1` once (needs internet). It stores `qrcode` and `jsQR` in `frontend/vendor/` and pins their SHA-256. The app itself never loads scripts from a CDN. See `docs/ENGINEERING-PASS-8.md` for what changed in Pass 8 and what is still unverified.

## Run in development

Place the two official core executables in the paths above, then run:

```powershell
npm install
npm run dev
```

## Build Windows installer

Run PowerShell from the project root:

```powershell
.\scripts\build-windows.ps1
```

The script verifies both core binaries, adds them as Tauri resources, and builds NSIS/MSI installers.

## Build portable package

After the release build:

```powershell
.\scripts\build-portable-windows.ps1
```

This creates `dist/TLC-LINK-Portable/` and `dist/TLC-LINK-Portable.zip`. WebView2 Runtime may still be required on the target Windows machine.

## Native command contract

- `runtime_info`
- `validate_core_config({ core, configJson })`
- `connect_core({ core, configJson })`
- `disconnect_core`
- `connection_status`
- `reset_network` (emergency: stop core + restore Windows proxy)
- `connect_core` now returns `process_running`, `proxy_applied`, `tunnel_verified`, `latency_ms`

The frontend uses Tauri's global bridge (`window.__TAURI__.core.invoke`).

## Added desktop menu features

The frontend now includes File, Settings, Tools, and Help menus. Profile export, JSON backup/restore, profile favorites, a preferences dialog, clipboard checks, quick guide/About, and website link are wired in the frontend. Note: Windows startup and tray checkboxes currently save preferences only; they do not enable native Windows startup or tray behavior. Clipboard checking on focus requires clipboard permissions. The frontend still loads QR libraries from CDN, so offline QR support is not bundled in this source snapshot.

## Current limitations (kept explicit)
- Ping is not a real probe yet; TLC LINK reports this instead of displaying randomized latency.
- QR display requires selecting a profile first and uses the current QR library availability; the fallback is an explanatory message, not a fake QR.
- Windows startup and system tray remain preferences-only in this foundation.
- Native connection functionality has not been tested in a Windows environment in this workspace.

## Expanded engineering direction
The product scope and staged acceptance criteria are maintained in `docs/IMPLEMENTATION-ROADMAP.md`; release verification items are in `docs/RELEASE-CHECKLIST.md`. This project remains a profile-import client, not a service/provider.

### Process lifecycle change in this revision
The native process manager now validates the requested replacement configuration before stopping an existing process, so a failed validation does not needlessly interrupt the current session. Startup detection polls for up to 900 ms and reports a short log tail if the core exits early. This is a code-level improvement, not a claim of Windows runtime testing.


## Pass 9 additions
- Run `cargo test --manifest-path src-tauri/Cargo.toml` and `node tests/frontend-checks.mjs` first.
- `scripts/fetch-vendor.ps1` (QR libraries) and `scripts/verify-cores.ps1` (core hashes) before packaging.
- Emergency recovery: **Tools → Emergency Network Reset**. Manual fallback: Windows *Settings → Network → Proxy* → turn "Use a proxy server" off.

## Experimental Windows TUN / DNS source pass

The current source includes an **experimental, unverified** Windows System TUN path for sing-box and a Cloudflare DoH-based DNS Guard. TUN requires launching the Portable app with **Run as administrator**; Xray remains System Proxy only. DNS Guard rejects profiles containing custom DNS rules rather than silently overriding them. **Kill Switch remains disabled and no firewall rules are changed.** This source pass has not yet been compiled with Rust or tested on Windows; follow `docs/ENGINEERING-PASS-10-TUN-DNS.md` before enabling it for normal users.
