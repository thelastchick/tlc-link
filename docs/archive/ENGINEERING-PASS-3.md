# Engineering Pass 3 — Process lifecycle hardening

## Scope of this pass
- Prevent orphaning a spawned core when `Child::try_wait()` fails during the startup monitor window.
- Clean up the candidate runtime config when stopping a previous process fails.
- Remove the candidate runtime config if creating/cloning the per-core log file fails.
- Keep startup validation-before-stop behavior: the candidate config is validated before the active session is stopped.
- Add unit tests for isolated runtime config files and malformed/non-object config input.

## Verified in this environment
- Rust source changes inspected for the startup-monitor error path and pre-spawn cleanup branches.
- JavaScript syntax checked with Node.
- Frontend HTML checked for duplicate IDs.
- Tauri/Cargo JSON checked.
- ZIP integrity checked after packaging.

## Not verified
- Rust compile and unit test execution: Cargo is not installed in this environment.
- Windows process launch, network routing, TUN, system proxy, DNS, Kill Switch, recovery, QR offline mode, subscription updates, and installer execution.
- QR dependencies cannot be vendored here because external network/DNS access is unavailable; CDN script references remain for a later offline-library pass.
