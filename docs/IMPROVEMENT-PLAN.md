# TLC LINK — Improvement Plan (Agent-driven)
**Started:** 2026-11-28  
**Policy:** Fail-closed, product honesty, no fake “working” states.

## Completed in this session

### P0 — Kill Switch hostname resolution (DONE)
- `extract_tunnel_endpoint` now resolves hostnames via OS DNS (`ToSocketAddrs`).
- Prefer IPv4, fall back to IPv6.
- When user requests Kill Switch and resolution/extraction fails → **hard fail** (stop core + TUN cleanup + emergency clear).
- Success message shows `host → resolved IP`.
- Unit tests added for literal IPv4/IPv6, proxy-tag preference, missing server.
- See `docs/CHANGE-REPORT-KS-HOSTNAME-2026-11-28.md`.

### P0 — KS service Status tracking (DONE)
- `tlc-ks-service` previously always reported `armed: false`.
- Now tracks armed state + generation with atomics and returns accurate Status.
- Documented remaining security limitation (open named-pipe DACL).

### P0 — Multi-upstream DNS Leak Guard (DONE)
- Bootstrap: UDP to 1.1.1.1 / 1.0.0.1 / 8.8.8.8 (literal IP, detour direct).
- Protected DoH: Cloudflare (primary by IP) + Google + Quad9 through proxy.
- Auto-inject `direct` outbound; `prefer_ipv4`; custom `dns.rules` still rejected.
- EN/FA UI strings updated.
- See `docs/CHANGE-REPORT-DNS-MULTI-UPSTREAM-2026-11-28.md`.

### P0 — KS startup reconcile (DONE)
- Corrupt journal → flagged + orphan WFP sweep via emergency_clear.
- Residual Armed/Blocked states → hard-block re-arm or clear on failure.
- Startup path triggers on corrupt **or** residual state.
- See `docs/CHANGE-REPORT-KS-RECONCILE-2026-11-28.md`.

### P1 — Live traffic meters (DONE)
- sing-box Clash API on 127.0.0.1:16789 (secret local)
- `traffic_snapshot` command + UI rates from successive totals
- Xray: shows unavailable (no fake numbers)

### P1 — Subscription refresh (DONE)
- Manual refresh fetches HTTP(S) subscription bodies
- Base64 / plain multi-link payloads via existing parser
- Auto-refresh on add; refresh button on each sub
- CSP allows https/http connect for fetch

### P1 — Latency IPv6 fix (DONE)
- `endpoint_tcp_latency` parses `[ipv6]:port` correctly

### P1 — URI converter (partial)
- VLESS `packetEncoding` / `packet-encoding` forwarded to sing-box.
- Allowlisted; unit test added.

### Packaging — RC preparation (IN TREE)
- `docs/RC-PREPARATION.md`, `docs/KNOWN-LIMITATIONS.md`, `docs/RELEASE-NOTES-0.2.0-rc.1.md`
- Version bumped to `0.2.0-rc.1`
- Portable script ships limitation docs
- Binary placeholders under `src-tauri/binaries/*`

## Next ordered work (current priority)

### P0 still open
1. **Independent KS service hardening**
   - Restrict named-pipe ACL to Administrators
   - Optional shared-secret auth
   - Reliable install/uninstall + auto-start after reboot
   - Code-signing story for the service binary

2. **System Proxy crash recovery edge cases**
   - Already good design; audit orphan-kill and “snapshot_unreliable” paths once more

### P1
3. **URI converter completeness (continued)**
   - Common VLESS Reality / XHTTP edge cases
   - Better SS base64 variants
   - Clearer error messages for unsupported params (already partially done)

5. **Minimal subscription support**
   - Fetch + parse base64 / plain list
   - Manual refresh only first (no scheduler yet)
   - Persist last-refresh time

6. **Real latency probe**
   - TCP connect or HTTP HEAD through the local inbound after connect
   - Never show random numbers

7. **Runtime DoH health probe after connect**
   - Confirm at least one protected upstream answers; warn or fail-closed if none do

### P2
7. TUN route/DNS cleanup hardening
8. Native tray + Windows startup registration
9. Offline QR fully bundled + size limits
10. Portable build + checksums + release checklist automation

## Explicit non-goals for now
- Claiming “production VPN” status
- Fake traffic stats or fake latency
- Supporting every exotic protocol before the core security paths are solid
