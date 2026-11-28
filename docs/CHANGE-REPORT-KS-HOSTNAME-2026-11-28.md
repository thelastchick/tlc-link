# Change Report — Kill Switch Hostname Resolution
**Date:** 2026-11-28

## Problem
`extract_tunnel_endpoint` previously required the outbound `server` field to be an IP literal.
Any profile that used a hostname (the majority of real-world subscription links) caused:
- Kill Switch to silently skip arming, or
- Only a warning string to be appended while the tunnel stayed up without protection.

This violated the product’s fail-closed promise when the user had explicitly enabled Kill Switch.

## Delivered

### 1. DNS resolution in `extract_tunnel_endpoint` (`lib.rs`)
- IP literals (IPv4 / IPv6) still take the fast path.
- Hostnames are resolved with a blocking `ToSocketAddrs` lookup.
- Preference order: first IPv4 address, then first IPv6 address.
- Returns `None` only when the outbound cannot be found or DNS fails.

### 2. Strict fail-closed policy when KS is requested
- If the user enabled Kill Switch and endpoint extraction / resolution fails →
  core is stopped, TUN interface is cleaned up, KS journal is emergency-cleared,
  and a clear multi-language error is returned.
- Previously this path only produced a soft warning while leaving the tunnel running.

### 3. Better user feedback on success
- On successful arm the detail string now includes both the original host and the
  resolved IP (`host → ip`) so the user can see what was actually permitted.

### 4. Unit tests
- IPv4 literal
- IPv6 literal
- Preference for `tag == "proxy"`
- Missing server → `None`

## Remaining known gaps (not claimed fixed)
- Resolution uses the system DNS resolver of the process. If the system DNS itself
  is already broken or poisoned, resolution can fail (correctly treated as fail-closed).
- No caching of resolved addresses across reconnects yet.
- No Happy-Eyeballs / dual-stack simultaneous connect; single preferred address is used.
- Service path (`tlc-ks-service`) already accepted an IP string; it benefits automatically
  from the new resolution performed in the UI process.
- Full Windows validation matrix (IPv6, sleep/resume, reboot, crash) still required
  before a public “verified” claim.

## Risk
Blocking DNS lookup runs under the existing `op` serialisation lock, so it cannot
interleave with another connect/disconnect. Worst-case delay is the OS resolver timeout
(typically a few seconds), which is acceptable for a connect operation.
