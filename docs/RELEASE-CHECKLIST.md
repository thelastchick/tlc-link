# TLC LINK Release Checklist — Pass 9

Rules: **PASS** only if actually executed with a real result. **FAIL** includes "feature not implemented".
**NOT TESTED** = could not be run in the authoring environment (no Windows, no Rust toolchain, no network, no core binaries).
Nothing here is assumed to work.

Environment of the authoring session: Linux sandbox, Node.js available, no `cargo`/`rustc`, no internet, no `sing-box`/`xray`.

## Mandatory tests (Section 12 of the Pass 9 brief)

| # | Test | Result | Notes |
|---|---|---|---|
| 1 | Successful build on Windows | NOT TESTED | Never compiled, not even on Linux. See "Known risks" in ENGINEERING-PASS-9.md. |
| 2 | App starts without cores, clear error | NOT TESTED | `AppError::CoreMissing` path exists in source. |
| 3 | sing-box connects in System Proxy | NOT TESTED | No binary available. |
| 4 | Xray connects in System Proxy | NOT TESTED | No binary available. |
| 5 | Previous proxy settings restored exactly | NOT TESTED | Logic covered by 14 unit tests with a fake registry — **written, not executed**; real registry untested. |
| 6 | Manual Proxy set beforehand | NOT TESTED | Unit test `manual_proxy_is_restored_exactly` written, not run. |
| 7 | PAC set beforehand | NOT TESTED | Unit test `pac_configuration_is_restored_exactly` written, not run. |
| 8 | Failed connect leaves Windows unchanged | NOT TESTED | Code path: probe fails → core stopped before any registry write. |
| 9 | Occupied port → clear error | NOT TESTED | `port_is_free` pre-check + unit test written, not run. |
| 10 | Kill core in Task Manager → proxy restored | NOT TESTED | Restore happens in the 2.5 s status poll; unverified. |
| 11 | Force-close TLC LINK → recovery on next start | NOT TESTED | Job Object + startup `recover_if_dirty`; unverified. |
| 12 | Windows restart recovery | NOT TESTED | |
| 13 | Successful connection in TUN | FAIL | Not implemented; TUN gated off. |
| 14 | TUN adapter disconnect/reconnect | FAIL | Not implemented. |
| 15 | Route and DNS after Disconnect (TUN) | FAIL | Not implemented. |
| 16 | Kill Switch and Emergency Reset | FAIL | Kill Switch not implemented. Emergency Reset (proxy + core) written, NOT TESTED. |
| 17 | IPv4 and IPv6 | FAIL | Not implemented for TUN/DNS; IPv6 server literal parsing has a unit test (not run). |
| 18 | DNS leak test | FAIL | Not implemented. |
| 19 | Import and QR fully offline | NOT TESTED | `frontend/vendor/` is empty; `fetch-vendor.ps1` not run (no network). |
| 20 | Protocol conversion | NOT TESTED | Unit tests exist (incl. new Pass 9 tests); `cargo test` not run. Real-core validation not run. |
| 21 | Subscription update, keep data on failure | FAIL | Subscription backend not implemented. |
| 22 | Real traffic statistics | FAIL | Not implemented. |
| 23 | No orphan processes | NOT TESTED | |
| 24 | No secrets in logs | NOT TESTED | UI-log redaction verified by `tests/frontend-checks.mjs` (**PASS** for that part only). Core log file contents not inspected. |
| 25 | Installer and Portable | NOT TESTED | Scripts not run. |

## Checks that were actually executed
| Check | Result |
|---|---|
| `node --check frontend/app.js` | PASS |
| `node tests/frontend-checks.mjs` (duplicate IDs, no remote origins, strict CSP, inert toggles, log redaction) | PASS |
| `cargo test` | NOT TESTED (no toolchain) |
| `cargo build --release` / `npm run build` | NOT TESTED |
| Bracket balance of new Rust files (crude) | PASS (not a substitute for compiling) |

## Packaging prerequisites (all open)
- [ ] Official sing-box and Xray downloaded, versions recorded, SHA-256 verified (`scripts/verify-cores.ps1`)
- [ ] `scripts/fetch-vendor.ps1` run, `vendor-hashes.json` reviewed
- [ ] Local fonts bundled (Vazirmatn / Inter / JetBrains Mono) — currently system fallbacks
- [ ] WebView2 requirement verified on clean Windows 10/11
- [ ] Code-signing status decided and documented
- [ ] SHA-256 of installer / portable published (`scripts/make-checksums.ps1`)
- [ ] Licenses / notices reviewed (`THIRD-PARTY-NOTICES.md`; sing-box is GPL-3.0-or-later — confirm distribution obligations)

## Release gate
Do **not** publish until rows 1–12, 16, 19, 20, 23, 24, 25 are PASS and TUN / Kill Switch / DNS Protection either work and pass 13–18 or stay disabled in the UI (as they are now).

## Kill Switch release gate (2026-11-27)

- [ ] Windows service/helper installer and authenticated IPC
- [ ] Persistent TLC-owned WFP provider/sublayer/filters
- [ ] IPv4 + IPv6 + DNS + UDP/QUIC direct-egress blocking
- [ ] Tunnel endpoint exception and zero-gap server switching
- [ ] Crash/reboot/service-restart reconciliation
- [ ] Explicit Disconnect release + exact recovery verification
- [ ] Windows VM packet-capture validation matrix in `KILL-SWITCH-IMPLEMENTATION-SPEC.md`

Until every item above is checked, `runtime_info.capabilities.kill_switch` must remain `false` and the UI toggle must remain disabled.
