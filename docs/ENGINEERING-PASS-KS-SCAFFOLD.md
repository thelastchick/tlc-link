# Engineering note — Kill Switch scaffolding (2026-11-28)

## What was delivered in source

1. **`kill_switch.rs`**
   - Full state machine matching the contract in `KILL-SWITCH-IMPLEMENTATION-SPEC.md`.
   - Versioned, atomic journal (`tlc-ks-journal.v1.json`) under the app config directory.
   - `begin_prepare` / `confirm_armed` / `begin_switch` / `on_unexpected_core_exit` / `begin_release` / `emergency_clear`.
   - Hard product gate: `capability_enabled()` always returns `false` until the WFP + service matrix is proven.

2. **Integration**
   - `AppState` owns a `Mutex<KillSwitch>`.
   - Startup: journal reconciled; leftover non-Disabled state triggers emergency clear and a startup notice.
   - Explicit Disconnect and process Exit call `begin_release`.
   - Emergency Network Reset includes a `firewall` step that always runs `emergency_clear`.

3. **Honesty**
   - Frontend strings still say “not available yet”.
   - `runtime_info.capabilities.kill_switch == false`.
   - No filter is written to the Windows Filtering Platform by this build.
   - Unit tests cover the gate and journal only.

## What is still required before enabling the capability

| Item | Status |
|------|--------|
| Independent signed Windows service owning filter lifetime | Missing |
| Authenticated local IPC (UI ↔ service) with ACLs | Missing |
| Real WFP provider / sublayer / transactional filters | Missing |
| Boot-time persistent block filters | Missing |
| Full validation matrix (IPv4/IPv6, DNS, crash, switch, reboot) on real Windows | Missing |
| `kill_switch: true` in `runtime_info` | Must stay false |

## License note

Patterns were studied from public documentation of Mullvad’s WFP design and sing-box TUN behaviour. No source code was copied from those projects. Any future WFP crate or third-party dependency must be reviewed for license compatibility before bundling.

## How to activate later (after real tests)

1. Implement WFP backend behind `wfp_backend_ready`.
2. Ship and install the helper service (portable UI may still require a one-time helper installer).
3. Run the mandatory matrix on Windows 10/11; record PASS results in `RELEASE-CHECKLIST.md`.
4. Only then flip `capability_enabled()` and the UI toggle.
