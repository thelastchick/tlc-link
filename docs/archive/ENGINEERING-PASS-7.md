# TLC LINK — Engineering Pass 7

## Scope delivered in this source pass

- Added a dedicated `network` module and a typed `NetworkMode` (`system-proxy` / `tun`).
- Added a TLC-managed local inbound on `127.0.0.1:2080` for sing-box (`mixed`) and Xray (`socks`) in System Proxy mode.
- Added a sing-box native TUN inbound with `auto_route`, `strict_route`, interface name, and a default DNS route when a DNS section is absent.
- Explicitly rejects TUN with Xray. This build does not pretend Xray has sing-box's native TUN implementation.
- Frontend now passes the selected network mode to native validation and connect commands.
- CSP now permits the existing QR libraries from jsDelivr; QR therefore still requires network access. Vendoring the libraries locally remains recommended.
- Added unit tests for mode-specific config generation.

## Important limitations — do not label this build release-ready

1. **Windows System Proxy is not automatically applied/restored yet.** The Core exposes a local mixed/SOCKS inbound at `127.0.0.1:2080`, but this pass does not edit Windows Internet Settings. Until Windows proxy integration and crash recovery are implemented, users must configure a compatible app/browser proxy manually. The UI log calls this out.
2. **TUN is configuration-level only until runtime validation on Windows.** sing-box must be packaged, run with the required privileges, and tested for adapter creation, routing, DNS, IPv6, and recovery. No claim of a verified full-system tunnel is made.
3. **Kill Switch and DNS Guard remain UI preferences, not enforced firewall/DNS policies.** They must remain disabled or clearly marked as not implemented in release builds.
4. **No Core executables are included.** Place trusted, version-pinned `sing-box.exe` and `xray.exe` in the expected binaries folders before packaging.
5. QR libraries remain CDN-loaded. CSP allows jsDelivr in this pass; offline operation requires vendoring and integrity pinning.
6. Build/test execution was not possible in the editing environment because Rust/Cargo and Windows runtime are unavailable. Run `cargo test` and Windows integration tests before distributing.

## Native command contract

`validate_core_config` and `connect_core` accept an optional `networkMode` string. Valid values are `system-proxy` (default) and `tun`.

## Next mandatory engineering tasks

- Implement Windows proxy registry backup/apply/restore with notification and crash-safe recovery.
- Validate sing-box TUN config against the exact pinned sing-box release and implement privilege elevation/adapter recovery.
- Add real end-to-end connectivity probes and real traffic counters.
- Implement firewall-backed Kill Switch only after rollback and recovery tests.
- Vendor QR libraries locally and remove CDN dependency.
