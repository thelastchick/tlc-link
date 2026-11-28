# Change Report — Multi-upstream DNS Leak Guard (DoH)
**Date:** 2026-11-28

## Problem
DNS Guard previously used a single Cloudflare DoH endpoint (`1.1.1.1`) for both
bootstrap and protected resolution. Networks that block Cloudflare, or transient
failures on that single path, caused total DNS failure while the tunnel appeared up.

## Delivered

### Bootstrap (always direct, literal IPs, UDP)
| Tag | Server | Role |
|-----|--------|------|
| `tlc-link-bootstrap` | 1.1.1.1:53 | Primary bootstrap |
| `tlc-link-bootstrap-cf` | 1.0.0.1:53 | Cloudflare secondary |
| `tlc-link-bootstrap-google` | 8.8.8.8:53 | Google fallback |

### Protected DoH (through proxy outbound)
| Tag | Server | Notes |
|-----|--------|------|
| `tlc-link-doh` | 1.1.1.1 (SNI cloudflare-dns.com) | Primary; IP literal → no extra resolve |
| `tlc-link-doh-google` | dns.google | `domain_resolver: tlc-link-bootstrap` |
| `tlc-link-doh-quad9` | dns.quad9.net | `domain_resolver: tlc-link-bootstrap` |

### Other
- `dns.final` = `tlc-link-doh`
- `dns.strategy` = `prefer_ipv4`
- Auto-inject `direct` outbound if missing (needed by bootstrap `detour`)
- Custom `dns.rules` still rejected when Guard is on (honesty)
- Unit tests updated for multi-server layout
- EN / FA UI strings updated to mention multi-upstream

## Design notes (from mature clients)
- Bootstrap must be reachable without the tunnel and without hostname lookup.
- DoH hostnames must themselves be resolved by bootstrap, not by the DoH server.
- Several independent operators reduce single-provider blocking risk.
- We still do **not** claim zero-leak without an independent DNS-leak test on the target network.

## Still open
- No runtime probe that DoH queries actually succeed after connect
- No user-selectable primary upstream yet
- System Proxy mode still has no DNS Guard path
- Full Windows acceptance matrix not executed
