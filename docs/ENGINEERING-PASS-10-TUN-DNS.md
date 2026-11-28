# TLC LINK — Engineering Pass 10: Windows TUN + DNS Guard

**Status: source implementation candidate. NOT compiled and NOT tested on Windows in this workspace.**

This pass is intentionally limited to the existing Portable source. The previous working source/archive must be kept as a rollback copy until a Windows build and the acceptance tests below pass.

## Implemented in source (pending Windows verification)

### System TUN (Sing-box only)
- TUN mode is available only on Windows and only with sing-box. Xray continues to use System Proxy.
- The frontend can select TUN while disconnected. Selecting TUN automatically selects sing-box.
- TUN connection requires an elevated process. The user must close TLC LINK and run `TLC-LINK.exe` using **Run as administrator**. This patch does not silently relaunch or elevate the app.
- Runtime config creates a dedicated `TLC-TUN` interface with IPv4/IPv6 addresses and explicit DNS addresses, enables `auto_route`, and sets `route.auto_detect_interface` to prevent the proxy outbound from looping back into the tunnel.
- Startup verification checks the TLC adapter is up, checks the two IPv4 split-default routes (`0.0.0.0/1`, `128.0.0.0/1`), and performs a bounded TCP reachability probe against multiple literal-IP endpoints.
- TUN cleanup only targets the interface named `TLC-TUN`; it does not reset global firewall policy or unrelated routes.
- If a previous `TLC-TUN` interface is present and the app is not elevated, startup shows a recovery notice and asks the user to reopen elevated and use Tools → Emergency Network Reset.

### DNS Leak Guard
- Requires System TUN and sing-box.
- Configures a modern HTTPS DNS upstream using Cloudflare DoH (`1.1.1.1`, TLS name `cloudflare-dns.com`).
- The DoH server uses the selected proxy outbound as its detour. The proxy hostname uses a fixed-IP bootstrap DoH resolver to avoid DNS recursion.
- `dns_mode: hijack` configures native interface DNS and DNS hijacking. When DNS Leak Guard is selected, `strict_route: true` asks sing-box to block ordinary port-53 DNS from non-TUN Windows interfaces.
- Existing custom `dns.rules` are rejected when DNS Leak Guard is selected. TLC LINK does not silently claim protection when custom DNS routing could override the managed path.
- The UI explains that Cloudflare DoH is used.
- The success message reports adapter, IPv4 route, TUN-interface DNS and TCP probe checks, and explicitly asks for an independent DNS-leak test. It does **not** claim that an external DNS-leak audit was performed.

### Kill Switch
- **Still NOT implemented and remains disabled in the UI/runtime capability flags.**
- No firewall rules are changed by this patch.
- A reliable Windows Kill Switch needs a separately designed and tested Windows Filtering Platform / firewall implementation with safe rollback and crash/reboot recovery. It is not simulated with a checkbox or a global firewall-policy change.

## Sing-box requirement

The TUN `dns_mode` and `dns_address` options were introduced in sing-box 1.14. The Windows workflow now fails if the official latest stable sing-box release is older than 1.14.0. The workflow still fetches the latest stable release, then records its version and checksum through the existing core verification step.

## Important limits

- No Rust toolchain, Windows runtime or Sing-box executable was available in the authoring environment. `cargo test`, Sing-box config validation, Wintun interface creation, Windows route installation, DNS behavior, and cleanup have **not** been executed here.
- The TCP probe is a basic reachability smoke test, not a speed test and not proof that every application is routed.
- DNS Guard's configuration and Windows per-interface DNS setup are checked at connect time, but the user must still perform an independent DNS-leak test on the target Windows/network combination before this feature is advertised as production-ready.
- Custom DNS routing rules are not compatible with DNS Guard in this pass; the app returns a clear error rather than deleting the user's rules.
- TUN requires the user to run the Portable app as Administrator. System Proxy mode remains available without elevation.
- Keep Kill Switch disabled until its own implementation and recovery tests exist.

## Required Windows acceptance tests before release

1. Build Portable with the official Sing-box stable binary (>= 1.14.0) and verify `sing-box check` accepts generated TUN + DoH config for a VLESS URI profile.
2. Launch TLC LINK elevated, select sing-box, enable System TUN, and connect with a known working profile.
3. Confirm `TLC-TUN` adapter status is Up and the IPv4 split-default routes use its interface index.
4. Open ordinary HTTP/HTTPS sites and compare public IP with System Proxy mode.
5. Enable DNS Leak Guard; verify the `TLC-TUN` interface receives `172.19.0.2` and that DoH requests work through the proxy.
6. Run an independent DNS leak test, including an IPv6-capable test where IPv6 is available. Do not mark DNS Guard production-ready until this passes.
7. Disconnect normally; confirm TLC-TUN routes and DNS settings are removed and ordinary internet access returns.
8. Force-close sing-box and force-close TLC LINK; verify startup recovery/Emergency Network Reset restores connectivity.
9. Test networks that block one or more public probe IPs; verify the multi-target probe does not fail just because one endpoint is blocked.
10. Confirm Xray + TUN is rejected with a clear message and System Proxy with either core continues to work.
11. Confirm Kill Switch remains disabled and no firewall policy is modified.

## Local checks performed in the authoring environment

- `node --check frontend/app.js` — passed.
- `node tests/frontend-checks.mjs` — passed.
- Rust compilation/tests — not run (Rust toolchain unavailable).
- Windows/TUN/DNS runtime tests — not run (Windows unavailable).
