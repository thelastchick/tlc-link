# Change Report — TLC LINK Kill Switch Scaffolding
**Date:** 2026-11-28  
**Scope:** Real Kill Switch architecture scaffolding (not activation)

## Summary
The project already documented that Kill Switch was unimplemented. This pass adds the production-oriented state machine, persistent journal, emergency recovery, and integration points required by `KILL-SWITCH-IMPLEMENTATION-SPEC.md`, while **keeping the capability disabled**.

No security feature is advertised as active. No WFP filters are written by this build.

## Files added
| Path | Purpose |
|------|---------|
| `src-tauri/src/kill_switch.rs` | State machine, journal, emergency clear, unit tests |
| `docs/ENGINEERING-PASS-KS-SCAFFOLD.md` | Engineering note and remaining work |
| `docs/CHANGE-REPORT-KS-SCAFFOLD-2026-11-28.md` | This report |

## Files modified
| Path | Change |
|------|--------|
| `src-tauri/src/lib.rs` | Module import, `AppState.kill_switch`, startup reconciliation, disconnect/release, exit release, Emergency Reset firewall step |
| `docs/KILL-SWITCH-IMPLEMENTATION-SPEC.md` | Status updated to SCAFFOLDING |
| `CHANGELOG-USER.md` | Entry for this pass |

## Behaviour guarantees of this build
1. `runtime_info.capabilities.kill_switch == false`
2. Frontend still shows “Kill Switch — not available yet”
3. `begin_prepare` / `begin_switch` are no-ops while capability is off
4. Emergency Network Reset always clears any leftover journal
5. Startup clears non-Disabled leftover journal and surfaces a notice
6. Existing System Proxy, TUN adapter cleanup, profiles, and UI are untouched

## What was intentionally **not** done
- No independent Windows service binary
- No real WFP provider/sublayer/filter installation
- No signed helper installer
- No claim of leak protection
- No flip of the UI toggle or capability flag

## How to complete (future work on real Windows)
1. Implement WFP backend (provider key `TLC_PROVIDER_KEY`, sublayer `TLC_SUBLAYER_KEY`) using transactional `Fwpm*` APIs or a reviewed MIT/Apache WFP crate.
2. Ship a least-privilege signed service that owns filter lifetime and boot persistence.
3. Authenticated local IPC between portable UI and service.
4. Execute the mandatory validation matrix in the SPEC; record PASS/FAIL in `RELEASE-CHECKLIST.md`.
5. Only after PASS results, set `capability_enabled() → true` and enable the UI control.

## License
No third-party VPN client source was copied. Reference architectures (Mullvad WFP design notes, sing-box TUN semantics) were studied from public documentation only.

## Build readiness
- Source is ready for `cargo test` / `cargo build` on a Windows MSVC host once the existing Pass-9 dependencies are present.
- Portable packaging scripts and GitHub Actions workflow were not altered by this pass.
