# Change Report — Kill Switch Startup Reconcile Hardening
**Date:** 2026-11-28

## Problem
- A corrupt on-disk journal was silently replaced with an empty in-memory journal,
  leaving any residual TLC WFP filters active with no UI awareness.
- Startup reconcile only ran when the journal was present *and* in a residual state;
  corrupt files never triggered a sweep.

## Delivered
1. `KillSwitch` tracks `journal_corrupt` when the file exists but cannot be parsed.
2. `reconcile_on_startup`:
   - On corrupt journal → `emergency_clear` (sweep TLC WFP objects + delete file) and report.
   - On residual Armed/Blocked/Reconciling/… → install hard block (fail-closed) or clear on failure.
3. App startup calls reconcile when `journal_was_corrupt()` **or** residual state is present.
4. Unit test: corrupt journal is flagged; Armed→Reconciling still works.

## Still open
- Named-pipe ACL still unrestricted (any local process can talk to the service).
- Service has no dedicated `HardBlock` command; reconcile uses in-process WFP.
- Full reboot / sleep matrix not executed on Windows hardware.
