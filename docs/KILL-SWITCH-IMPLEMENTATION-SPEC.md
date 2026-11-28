# TLC LINK — Windows TUN Kill Switch implementation contract

Status: **SCAFFOLDING IN SOURCE — capability must remain `false`**

The state machine, journal, emergency recovery, and integration points are present in
`src-tauri/src/kill_switch.rs` and wired into connect / disconnect / reset_network /
startup / exit. The real WFP filter engine and signed independent Windows service are
**not** present. `runtime_info.capabilities.kill_switch` stays `false`.

This document is the engineering contract for the requested connection-scoped Kill Switch. It is not evidence that traffic is currently protected.

## Required user-visible behavior

1. Scope: Windows + sing-box TUN only. System Proxy mode must never advertise full-device leak protection.
2. Arm only after the tunnel endpoint (server IP:port/protocol) has been resolved and the TUN adapter/routes have been verified. The endpoint must remain reachable outside the tunnel.
3. While armed, block non-tunnel egress on every physical interface for IPv4 and IPv6, including DNS, UDP, TCP, QUIC and applications that ignore system proxy settings. Permit loopback and the minimum Windows networking required by the selected policy. Permit only the authenticated tunnel transport to its pinned endpoint outside TUN. Do not permit general direct traffic.
4. Server switch is a protected transaction: retain the old endpoint exception while the new core is prepared; add/verify the new endpoint exception before removing the old one. Never create an unprotected gap.
5. Unexpected core exit, UI crash, Task Manager termination, service restart, sleep/resume and adapter flap must leave the block in force until a healthy tunnel is re-established or the user explicitly disables protection.
6. Intentional Disconnect is a separate authenticated request: stop tunnel, remove only TLC-owned filters, verify removal, then return control to normal networking. If removal cannot be verified, report that the device may remain offline and offer a narrowly-scoped recovery action.
7. If setup or verification fails, fail closed: do not claim connected/protected and do not silently fall back to direct internet.

## Required architecture (before enabling the UI)

- A separately installed, signed, least-privilege Windows service owns the protection lifecycle. The portable UI communicates with it over an authenticated local IPC channel with ACLs restricted to the interactive user and service identity.
- The service owns persistent Windows Filtering Platform (WFP) filters. UI process lifetime must not own the filters or their lifetime. Use a dedicated TLC provider/sublayer and stable filter identifiers so recovery can enumerate and remove only TLC-owned objects.
- The service persists a small versioned transaction journal (state, tunnel endpoint, interface identity, filter IDs, generation). Journal writes must be atomic and ACL-protected. On boot/restart, reconcile journal against actual WFP state before allowing traffic.
- Filter installation/removal must be transactional. Verify effective filters and tunnel reachability before reporting Armed. A failed partial transaction must either roll back to the prior known-safe generation or remain blocked and expose recovery.
- The helper installer is a prerequisite even though the main application remains Portable. A portable executable alone cannot guarantee boot-time or post-crash enforcement.

## State machine

`Disabled -> Preparing -> Armed -> Switching -> Armed`

`Armed -> Releasing -> Disabled` only for an explicit user Disconnect/disable request.

`Preparing/Switching` failures must not enter Disabled automatically. Unexpected core exit from Armed enters `BlockedWaitingForTunnel`; it must not remove WFP filters. Service restart/boot enters `Reconciling` and remains blocked until the journal and filters agree.

## Integration points in current source

- `src-tauri/src/lib.rs::connect_core`: arm only after `wait_until_ready` and `verify_routed_connectivity`; for server switching, preserve old generation while preparing new one.
- `src-tauri/src/lib.rs::disconnect_core`: request explicit release only after the core has stopped; verify release before returning success.
- `src-tauri/src/lib.rs::reset_network`: add a separate TLC-filter reconciliation/removal step; never remove unrelated firewall/WFP state.
- `runtime_info`: `kill_switch` remains `false` until service install, authenticated IPC, WFP implementation, boot recovery and Windows integration tests all exist.
- `frontend/app.js`: toggle remains disabled; do not persist a fake enabled state.

## Mandatory Windows validation matrix

- IPv4 and IPv6 direct TCP/UDP/DNS blocked while Armed; same traffic succeeds through TUN.
- DNS over UDP/TCP, DoT, DoH, QUIC and browser proxy-bypass cases.
- Kill core process, kill UI process, crash UI, stop/restart service, reboot, sleep/resume, unplug/reconnect network adapter.
- Switch between two server endpoints while packets are flowing; assert no direct-egress window.
- Invalid endpoint, DNS resolution failure, unreachable server, WFP API failure, insufficient privileges, corrupted journal and disk-full conditions.
- Explicit Disconnect restores the exact pre-existing network behavior and removes only TLC-owned filters.
- Emergency recovery from service/UI, with a tested offline recovery path.

No Kill Switch release may be marked complete based only on Rust unit tests or a successful CI compile. The above tests require disposable Windows VMs plus packet capture on the physical interface.
