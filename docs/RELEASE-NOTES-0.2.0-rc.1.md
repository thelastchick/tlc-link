# TLC LINK 0.2.0-rc.1 — Release Candidate notes

**Status:** Release Candidate (Windows x64 Portable)  
**Not:** production feature-parity with v2rayN / Hiddify / Clash Verge Rev

## Highlights since 0.1.x engineering drops

- Real WFP Kill Switch path (TUN + elevation), fail-closed on arming failure
- Hostname resolution for KS endpoint (prefer IPv4)
- Startup reconcile + corrupt journal orphan sweep
- Multi-upstream DNS Leak Guard (Cloudflare / Google / Quad9) with bootstrap resolvers
- Live upload/download meters for sing-box (local Clash API)
- Manual subscription HTTP(S) refresh
- TCP latency probe with IPv6 `[addr]:port` parsing
- VLESS `packetEncoding` forwarded when present
- Honest capabilities flags (traffic / subscriptions)

## How to install (Portable)

1. Unzip `TLC-LINK-Portable-0.2.0-rc.1.zip`
2. Run `TLC-LINK.exe` (WebView2 required)
3. Prefer **System Proxy** for first tests
4. Use **Tools → Emergency Network Reset** if anything looks stuck

## Recovery

| Symptom | Action |
|---------|--------|
| No internet after crash | Disconnect, or Emergency Network Reset |
| KS still blocking | Emergency Network Reset (admin) |
| Proxy sites broken after exit | Emergency Network Reset |

## Tester focus

See `docs/RC-PREPARATION.md` section 5 and `docs/KNOWN-LIMITATIONS.md`.

## Build machine only

```powershell
# after placing official cores + vendor fetch
BUILD-WINDOWS.bat
```
