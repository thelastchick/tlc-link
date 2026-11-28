# TLC LINK — Development Roadmap (Expanded)

Product scope: TLC LINK is a desktop client that lets users import and manage their own profiles/configurations. It is not a provider service and does not bundle user credentials or demo endpoints.

## Stage 0 — Scope and design freeze
- Preserve the existing space/gold visual identity and current layout.
- Keep feature work focused on reliability, configuration management, diagnostics, and Windows packaging; no redesign unless an accessibility or bug fix requires it.
- Never ship demo profiles, fabricated connection states, or randomized measurements.

## Stage 1 — Native foundation and process lifecycle (priority)
- Tauri/Rust command validation and clear frontend error handling.
- Validate replacement profile before disrupting the current session.
- Track child process state; surface early exits and log excerpts.
- Ensure stop waits for process termination and app exit cleanup is handled.
- Use application-scoped config/data paths; document sensitive-data handling.
- Test repeated start/stop, invalid config, missing binary, core crash, app restart.

Acceptance gate: a failed attempt to start a new profile must not silently destroy an existing working session; process state must not be represented as a verified network tunnel.

## Stage 2 — Import and profile data quality
- Import through paste, clipboard, file, image QR, and text QR payload.
- Parse URI and native JSON formats with explicit detection and validation feedback.
- Prevent accidental duplicate imports; allow user-confirmed duplicates when desired.
- Profile metadata: display name, protocol, source, added/updated timestamps, favorite, group, and subscription association.
- Never store subscription URLs or profile credentials in logs.
- Test malformed, oversized, empty, and mixed-format inputs.

## Stage 3 — Core compatibility and configuration validation
- Maintain a documented compatibility matrix by core and format.
- Separate parser tests from engine validation tests.
- Report unsupported features instead of silently dropping fields.
- Validate generated configuration with the selected engine's own validator.
- Test against pinned engine versions and record version/build provenance.

## Stage 4 — Windows integration and recovery
- System proxy lifecycle with snapshot/restore and failure recovery.
- TUN only after adapter, privilege, route, DNS, and crash recovery tests exist.
- DNS changes must be reversible and tracked.
- Do not label Kill Switch as active unless firewall rules are installed and independently verified; provide rollback/recovery.
- Handle app crash, forced termination, reboot, permission denial, and disconnect.

## Stage 5 — Professional usability
- Subscription refresh, schedule, cancellation, and last-refresh status.
- Real latency/connectivity checks with timeout and clear methodology; never fabricate numbers.
- QR encode/decode with offline-bundled library and payload size handling.
- Structured logs with redaction; user-controlled export of diagnostic bundle.
- Backup/restore schema versioning, validation, and safe failure.
- Tray/startup preferences only after native behavior is implemented and tested.
- Updater with signature verification and rollback strategy.

## Stage 6 — Release engineering and security
- Windows 10 and Windows 11 clean-install test matrix.
- Installer and portable builds; verify uninstall and upgrade behavior.
- Code signing when a valid signing certificate is available; unsigned builds clearly labeled.
- SHA-256 checksums, licenses/notices, release notes, and pinned dependencies.
- Security review of Tauri capabilities, IPC input validation, resource paths, logging, and secrets storage.
- Test crash/recovery and network restoration before public release.

## Release gates
1. **Engineering build:** compiles, tests pass, no demo behavior.
2. **Release Candidate:** tested on real Windows machines, known limitations listed.
3. **Public Release:** signed (if available), packaged, checksummed, clean-install/upgrade/uninstall verified, and recovery behavior tested.

A process being alive is not proof that traffic is routed or a connection is healthy. All UI and release notes must distinguish process state from connectivity verification.

## Latest pass — process lifecycle reliability
This pass prioritizes the desktop-native supervisor over new UI. Candidate config files are isolated per validation/start attempt, the active runtime config is tracked for cleanup, logs are rotated once, and the frontend polls the native process state to reset a stale UI state after a core exit. These changes do not claim that a tunnel or traffic flow is verified. Windows tests and Rust compilation remain required.
