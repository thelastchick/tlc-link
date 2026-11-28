# TLC LINK — Changelog

## Kill Switch scaffolding (2026-11-28, source only)
- New module `src-tauri/src/kill_switch.rs`: state machine (Disabled → Preparing → Armed → Switching → Releasing / BlockedWaitingForTunnel / Reconciling), versioned atomic journal, emergency clear.
- Wired into `disconnect_core`, `reset_network` (firewall step), app exit, and startup recovery.
- `runtime_info.capabilities.kill_switch` remains **false**. No WFP filters are installed; no protection is claimed.
- Unit tests for capability gate, journal round-trip, and emergency clear (pure Rust, no Windows required).
- Real WFP engine + independent signed service remain required before the capability can be enabled. See `docs/KILL-SWITCH-IMPLEMENTATION-SPEC.md`.

## Pass 9 (source only, untested on Windows)
- System Proxy is now transactional: exact snapshot/restore (absent vs 0 preserved), verified registry read-back, corrupt state-file detection, no change to Windows before a real HTTP + HTTPS-tunnel probe passes.
- Server/mode switch restores Windows first, then swaps cores; Disconnect and exit restore before stopping the core.
- Port-in-use detection before start (fixes a case where a foreign listener could satisfy the probe).
- Core is tied to the app via a Windows Job Object; orphan cleanup checks the executable path, not just the PID.
- New: Tools → Emergency Network Reset with per-step report.
- Protocol converter rejects unsupported parameters instead of dropping them.
- Kill Switch, DNS Protection and TUN switches are disabled ("Coming Soon") until implemented.
- Stricter CSP, no Google Fonts, inline script/style moved to files, secrets masked in the log panel.
- Fixed build blockers: missing `[lib]` target and app icons.

## Earlier history — Desktop Foundation update

This update keeps the existing mockup and adds:

- A dedicated **Paste / پیست** button inside the import dialog. It reads clipboard text into the import box so users can review before saving. Keyboard Ctrl+V remains available.
- The existing quick clipboard-import button continues to parse recognized clipboard configurations directly, or opens the import dialog for review if it cannot parse the content.
- Replaced the fake QR pattern with a genuine QR canvas generator using QRCode, and added QR decoding for uploaded image files and clipboard images using jsQR.
- QR utility scripts are currently loaded from CDN, so QR creation/scanning requires an internet connection to load these scripts. Offline vendoring is a follow-up before a fully offline release.

This is a source-package update, not a signed/tested Windows installer. The app's network core, configuration conversion coverage, subscriptions, latency checks, TUN/routing, Kill Switch, and DNS protection still need implementation and testing before being advertised as production features.

## Desktop menu and convenience features (added in this update)
- Added File, Settings, Tools, and Help menu strip.
- File menu: import config, paste from clipboard, import file, export selected profile, export backup, restore backup.
- Preferences dialog: clipboard monitor toggle, saved Windows-start preference (not wired to OS startup), saved tray preference (tray integration not implemented), and confirm-before-clear option.
- Tools menu: existing ping-all control, jump to logs, clipboard check.
- Help/About dialog and website link.
- Favorite toggle (star) for each profile, persisted with profiles.
- JSON backup/restore includes profile list, subscription entries, selected profile, core/routing and toggles, and app preferences. Backup can contain sensitive configuration strings; protect it.
- Clipboard monitor checks clipboard on app focus only when enabled and opens import for recognizable config text. Clipboard access may require OS/browser permission.

## Repair pass (2026-11-26)
- Removed random placeholder latency values from imported profiles.
- Changed Ping All to clearly report that real latency testing is not available yet; it no longer invents numbers or marks profiles with fake results.
- QR display no longer creates a dummy demo profile when no profile is selected; it prompts the user to select a saved profile.
- Expanded the Profile Manager summary to include profile, favorite, and subscription counts.
- Checked the embedded frontend JavaScript syntax with Node.js and validated the Tauri/package JSON files.

## Expanded development pass (2026-11-26)
- Roadmap expanded around the requested priority: infrastructure/process reliability first, config/import quality, Windows network recovery, professional features, then release gates.
- Added a detailed release checklist.
- Native process manager now validates the new config before stopping a current process and checks early core exits during a bounded startup window.
- This pass has not been compiled with Rust or exercised on Windows in this environment.

## 2026-11-26 — Core supervision hardening (work-in-progress)
- Config files for validation and runtime are now written with unique names to avoid one profile's config overwriting another core's config file.
- `validate_json` deletes its temporary config after validation.
- Failed validation, failed process launch, and quick startup exit clean up the candidate config file.
- The running core's config file is tracked and removed on normal disconnect, detected exit, or best-effort app shutdown.
- Core log rotates to one previous file before a new run to prevent unbounded log growth.
- Native process state is periodically polled by the desktop UI; if the process exits, the visible connection state is reset instead of remaining stale.
- Added Rust unit tests for core name parsing and executable/config naming. Tests are added but not run in this environment (Cargo unavailable).
- JavaScript syntax check passed for all inline script blocks; HTML duplicate-ID check passed.

## Engineering Pass 2 (2026-11-26)
- Runtime config files use exclusive creation with a monotonic sequence suffix to avoid accidental overwrite on rapid operations.
- Windows build script now restores the original Tauri configuration after build success or failure.
- Portable build script accepts common executable output names and cleans only its package output artifacts.
- Validation note: JS syntax passed; Rust and Windows builds were not available in this environment.

## Engineering Pass 3
- Hardened startup-monitor error handling: if polling a launched core fails, the child is explicitly killed and waited before returning the error, preventing an orphan process.
- Removed runtime-config leftovers when replacing an existing session fails to stop, and when per-core log creation/duplication fails.
- Added unit tests for unique temporary runtime config paths and rejecting invalid/non-object JSON.
- Offline QR dependency packaging remains blocked in this environment by unavailable external DNS/network; CDN scripts were not falsely reported as localized.

## Engineering Pass 4 — initial protocol conversion
- Added a Rust URI conversion module for basic VLESS, VMess, Trojan, Shadowsocks, SOCKS5, and sing-box Hysteria2 support.
- Connect/validate can now receive a single URI and convert it to the selected core JSON before handing it to the engine.
- Added converter unit tests (not run: Rust toolchain unavailable in this environment).
- This is an initial conversion slice; protocol coverage, subscriptions, QR offline support, Windows networking, Kill Switch, and recovery remain unfinished.

## Engineering Pass 5
- Connect action now calls the Rust URI converter for URI-based profiles, while retaining direct JSON profile support.
- Generated/imported JSON is validated by the selected core before launching.
- Improved percent-decoding to use UTF-8 bytes correctly.
- Fixed a malformed SOCKS credential check in the Rust converter.
- Not verified by Cargo or Windows runtime: Rust toolchain is unavailable in this environment.


## Engineering Pass 5
- Live Console Logs starts collapsed and opens automatically on error logs.
- Added clear-log action, copy-report-to-clipboard action, and visible log-entry count.
- Existing log level colors (success/info/warning/error) preserved.
- Live traffic rate/cumulative usage, verified tunnel duration, and real latency remain pending backend integration.

## Engineering Pass 6
- Localized log actions and copy-report labels/messages across all nine app languages.
- Added clipboard fallback using a temporary selectable textarea when the modern Clipboard API fails.
- Corrected common raw Shadowsocks URI parsing and stricter Base64 handling.
- Added Xray Reality stream-settings mapping and Rust test cases.
- Real connection traffic stats and real ping remain unimplemented; Rust/Windows runtime checks could not be run in this environment.

## Engineering Pass 7 — Network mode foundation
- Added System Proxy local inbound generation and sing-box TUN inbound generation.
- Added mode-aware native validation and connection startup.
- TUN is explicitly rejected for Xray rather than silently generating an invalid mode.
- Windows system proxy auto-configuration, crash-safe restoration, firewall Kill Switch, end-to-end traffic verification, and offline QR assets are still pending; this is an engineering preview, not a production release.

## Engineering Pass 8
- Connect now applies the Windows System Proxy only after a real request succeeds through the proxy; Disconnect, exit, core crash and next launch restore your previous settings.
- "Connected" is shown only when traffic through the proxy was verified.
- Fixed VMess links containing `/`, passwords containing `+`, and several silently-dropped link parameters.
- QR libraries are local (run `scripts/fetch-vendor.ps1` once). Not yet tested on Windows.

## Kill Switch real WFP + service (2026-11-28)
- Real Windows Filtering Platform backend (persistent ALE AUTH_CONNECT V4/V6 block + endpoint permit).
- Independent helper binary `tlc-ks-service` with named-pipe protocol; optional install via `scripts/install-ks-service.ps1`.
- UI prefers the service when running; falls back to in-process elevated WFP for Portable use.
- Kill Switch toggle enabled when capability is present; only arms in TUN mode when the user enables it.
- Unexpected core exit keeps the block (fail closed). Explicit Disconnect / Emergency Reset releases TLC-owned objects only.
- System Proxy mode never arms Kill Switch.
