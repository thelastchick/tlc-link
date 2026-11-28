# TLC LINK Engineering Pass 2

## Changes in this pass
- Runtime config file creation now uses exclusive `create_new` writes and a process-local sequence number, so rapid successive writes cannot overwrite a file with the same timestamp.
- The Windows installer build script restores the original `tauri.conf.json` in a `finally` block even if the build fails; temporary resource injection no longer permanently changes the config.
- The portable build script checks both common executable names, validates required core files, and only removes its own output folder/ZIP before rebuilding.
- Portable package README now refers to the actual copied executable (`TLC-LINK.exe`).

## Validation performed in this environment
- JavaScript syntax check: passed for the inline script.
- Tauri configuration JSON parse: pending / not run in this pass.
- Rust compilation and unit tests: not run (Rust toolchain unavailable).
- Windows installer/portable builds: not run (Windows toolchain/runtime unavailable).

## Known remaining release blockers
- QR libraries are still loaded from CDN (`qrcode` and `jsQR`); offline QR operation is not yet achieved.
- Config-link parsers/conversion to core-specific JSON remain incomplete; this pass did not claim or implement protocol conversion.
- System proxy/TUN, DNS, routes, kill switch, real latency testing, subscription refresh and end-to-end connectivity have not been validated.
- Core executables are not included in the source archive; Windows packaging requires official binaries placed in the expected folders.
- The app should be tested on a clean Windows 10/11 installation before release.
