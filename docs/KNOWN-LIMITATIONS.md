# TLC LINK — Known Limitations (RC target)

**Audience:** release packaging and testers  
**Honesty rule:** do not claim a capability that is not listed as supported below.

## Supported (intended for RC)

| Area | Scope |
|------|--------|
| Platform | Windows 10/11 x64 only |
| Cores | Official `sing-box.exe` and `xray.exe` beside the app |
| System Proxy | Apply + restore snapshot; Emergency Network Reset |
| TUN | Experimental, sing-box only; requires Administrator; not Windows-release-verified |
| DNS Leak Guard | Experimental; TUN + sing-box only; requires independent DNS-leak testing |
| Kill Switch | Disabled; WFP/service scaffolding only; not available to users until release gates pass |
| Profiles | Paste JSON or common URIs (VLESS / VMess / Trojan / SS / Hysteria2 / SOCKS) |
| Subscription | Manual HTTP(S) refresh; no auto schedule yet |
| Traffic meters | Live up/down rates for **sing-box** (Clash API loopback) |
| Latency | TCP connect time to `host:port` (not ICMP, not full tunnel RTT) |
| QR | Offline vendor libraries only (`frontend/vendor`) |

## Explicitly limited or unsupported

1. **Not a multi-platform product** — macOS / Linux / mobile are out of scope for this RC.
2. **TUN + Kill Switch are advanced** — may fail without Admin; recovery is Emergency Reset / Disconnect.
3. **Xray TUN** — not implemented; use System Proxy with Xray.
4. **Traffic stats for Xray** — unavailable (UI shows unavailable, no fake numbers).
5. **TUIC / WireGuard / full XHTTP** — rejected or incomplete rather than silently faked.
6. **Clash rule providers / complex rule sets** — not a Clash Verge replacement.
7. **Independent KS service** — scaffold exists; pipe ACL and full boot ownership still hardening.
8. **Code signing / auto-update** — not included in unsigned RC builds.
9. **Hostname-only profiles + Kill Switch** — resolution is attempted; failure is fail-closed (no silent “armed”).
10. **Sleep / hibernate / multi-homed edge cases** — must be verified on real hardware before calling production.

## Experimental / gated features

- Windows TUN and DNS Leak Guard are experimental and require real Windows acceptance testing; do not describe them as production-ready.
- Kill Switch is disabled in runtime capabilities and the UI until the independent service, authenticated IPC, boot recovery, and packet-capture validation matrix are complete.
- Live traffic counters are available only for sing-box through its loopback Clash API; Xray statistics are unavailable.

## Safety behaviours (by design)

- Core process running ≠ tunnel verified.
- Kill Switch failure while requested → stop core + cleanup (fail closed).
- Unexpected core exit with KS armed → stay blocked until Disconnect / Emergency Reset.
- Corrupt KS journal → sweep TLC WFP objects rather than trust unknown state.
- Unknown URI parameters are rejected instead of silently dropped.

## What “RC” means

Release Candidate means: portable package builds on Windows, System Proxy path is exercised on real machines, recovery paths work, and limitations above are published with the zip. It does **not** mean feature parity with v2rayN / Hiddify / Clash Verge Rev.
