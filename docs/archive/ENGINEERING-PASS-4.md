# TLC LINK — Engineering Pass 4

## Scope delivered in this pass
- Added a Rust URI-to-core configuration conversion module and Tauri command `convert_profile_uri`.
- Connect and validate commands now accept either an existing JSON core config or a single supported protocol URI and convert the URI before passing it to the core.
- Basic conversion implemented for VLESS, VMess, Trojan, Shadowsocks URI (base64 method:password@host:port form), SOCKS5, and Hysteria2 for sing-box.
- VLESS basic TLS, Reality fields, and WS/GRPC/TCP/HTTP transport parameters are represented in sing-box output; Xray output has basic VLESS/VMess/Trojan/Shadowsocks mappings.
- Added unit tests for basic VLESS sing-box, Trojan Xray, Shadowsocks, and rejection of unsupported schemes.

## Known limits / risks
- This source change is not compiled in this environment (Cargo/Rust toolchain unavailable). Rust tests are added but not run.
- Core executables are not bundled in the source ZIP, so real core validation/startup cannot be performed here.
- Conversion is a first-pass subset, not complete URI-spec coverage. Plugin options, less-common transport/security combinations, SS plugin syntax, IPv6 edge cases, and some Xray TLS/Reality mapping details may not be preserved.
- Xray conversion for Hysteria2, TUIC, and SOCKS is not implemented. WireGuard URI/profile conversion is not implemented.
- Subscription retrieval/update, offline QR libraries, Windows TUN/System Proxy/DNS/Route, real Kill Switch, and crash-time network recovery remain outstanding.
- Do not treat a started core process as proof of a functioning VPN tunnel. Existing UI correctly warns that tunnel verification is pending.

## Checks performed
- JSON files parsed with Python.
- Existing HTML inline JavaScript syntax checked with Node.
- Duplicate HTML IDs checked.
- ZIP integrity checked after packaging.
- Rust compilation and unit tests: not run (toolchain absent).

## Engineering Pass 5 — UI conversion integration

- The Connect action now attempts to parse profile content as JSON first.
- If the selected profile is a URI, it invokes `convert_profile_uri` for the currently selected core.
- Both imported JSON and converted URI configs are passed through `validate_core_config` before `connect_core`.
- Percent-decoding in the converter now decodes bytes and then UTF-8, rather than treating each byte as a Unicode character.
- Fixed an invalid character literal in the SOCKS5 credential check discovered during review.


### Limits and validation
- Conversion still has incomplete protocol coverage (notably TUIC, WireGuard, SOCKS5 for Xray, and Hysteria2 for Xray), and VLESS Reality/transport variants require further fidelity review.
- Rust compilation and runtime tests could not be run in this environment because Cargo/Rust are not installed. This pass is a source-level patch, not a verified Windows build.
