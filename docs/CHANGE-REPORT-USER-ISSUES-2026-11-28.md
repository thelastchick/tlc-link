# TLC LINK — Source Review and Corrections

Date: 2026-11-28

## Corrected in this source pass

- Removed silent automatic switching from Xray to sing-box when the user enables TUN or DNS Leak Guard. TUN/DNS now reject the incompatible Xray selection and preserve the user's chosen core.
- Kept DNS Leak Guard's real dependency: when sing-box is selected, enabling DNS Leak Guard also enables its required TUN mode. Disabling DNS Guard does not silently disable TUN if the user has left TUN selected.
- Set the Kill Switch runtime capability to disabled and made the native connect command reject attempts to request it. The existing WFP/service code is scaffolding and is not advertised as a working protection feature until the documented Windows release gates pass.
- Made traffic polling single-flight, discarded late responses from previous connection sessions, and kept the statistics panel visible with an explicit unavailable state during transient local API failures. Xray continues to report statistics unavailable rather than fabricated values.
- Updated Routing mode feedback: Direct / Global / Rule selection is stored for the next connection. It is not hot-applied to an already-running core in this build.
- Updated release documentation to distinguish experimental TUN/DNS paths from verified release capabilities.

## Important limitations not represented as fixed

- Xray TUN is not implemented; Xray remains System Proxy only.
- Kill Switch remains disabled; WFP scaffolding is not sufficient for a production claim.
- TUN and DNS Leak Guard remain experimental until validated on Windows 10/11 with official sing-box binaries, administrator elevation, route/DNS checks, disconnect recovery, and independent DNS-leak tests.
- Traffic statistics are available only for sing-box via the local Clash API. Xray statistics are not implemented.
- Protocol URI conversion intentionally rejects unsupported parameters and protocols; not every free configuration can be supported.

## Checks performed in this workspace

- `node --check frontend/app.js` — PASS
- `node tests/frontend-checks.mjs` — PASS
- Rust compilation/tests — NOT RUN (Rust/Cargo toolchain unavailable in this environment)
- Windows TUN / DNS / WFP / routing runtime tests — NOT RUN (requires Windows test machine and official core binaries)

This is a source-level correction pass, not a verified Windows release.
