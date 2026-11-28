# Change Report — Real WFP Kill Switch
**Date:** 2026-11-28

## Delivered

### WFP backend (`kill_switch.rs` / `wfp` module)
- `FwpmEngineOpen0` non-dynamic (persistent) session
- TLC provider + high-weight sublayer registration
- Block-all filters on `ALE_AUTH_CONNECT_V4` and `ALE_AUTH_CONNECT_V6`
- Permit filters for pinned server IP:port (and previous endpoint during switch)
- Transactional install/remove
- Enumerate-and-delete by provider key for emergency clear
- Hard-block re-arm on startup reconcile (fail closed)

### Integration
- `runtime_info.capabilities.kill_switch = true` on Windows
- After TUN verify: extract outbound server IP from config → `begin_prepare`
- Fail-closed: if arming fails, stop core + cleanup TUN + emergency clear
- Unexpected core exit: stay blocked (`BlockedWaitingForTunnel`), do not restore internet
- Explicit Disconnect / Exit / Emergency Reset: release only TLC objects
- Startup: reconcile journal → hard block or clear

### Frontend
- EN / FA / AR strings updated: Kill Switch is no longer “not available yet”

### Cargo
- `windows-sys` features: `Win32_NetworkManagement_WindowsFilteringPlatform`, `Win32_System_Rpc`

## Requirements to operate
1. Windows 10/11 x64
2. Run TLC LINK elevated (Administrator) for TUN + WFP
3. sing-box TUN mode
4. Outbound `server` field must be a literal IP (not hostname) for the permit filter; hostname-only profiles get a warning and KS is not fully armed

## Still recommended before public “production” claim
- Full validation matrix on real hardware (IPv4/IPv6, DNS, crash, switch, reboot, sleep)
- Independent signed service for boot-time enforcement (current design uses persistent WFP filters from the elevated UI process)
- Hostname → IP resolution path before arming when server is a domain
- Code signing of the binary

## License
No third-party VPN client source was copied. WFP usage follows public Microsoft API documentation.
