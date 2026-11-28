# TLC LINK — RC Preparation Guide

**Target tag:** `v0.2.0-rc.1`  
**Package name:** `TLC-LINK-Portable-0.2.0-rc.1.zip`

## 0. One-time machine setup (Windows build PC)

- [ ] Windows 10/11 x64
- [ ] Rust stable + `x86_64-pc-windows-msvc`
- [ ] MSVC Build Tools + Windows SDK
- [ ] Node.js LTS
- [ ] WebView2 Runtime
- [ ] Git

```powershell
rustc -V
cargo -V
node -v
npm -v
```

## 1. Place official cores

```
src-tauri/binaries/sing-box/sing-box.exe
src-tauri/binaries/xray/xray.exe
```

- Official upstream builds only
- Record versions + SHA-256 in the release notes
- Run: `powershell -File scripts/verify-cores.ps1` (if present)

## 2. Vendor QR (offline)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/fetch-vendor.ps1
```

Confirm:

```
frontend/vendor/qrcode.min.js
frontend/vendor/jsQR.js
```

## 3. Build

```powershell
# Full path (installer + portable)
BUILD-WINDOWS.bat

# Or portable only after a release cargo/tauri build:
powershell -ExecutionPolicy Bypass -File scripts/build-portable-windows.ps1
```

Expected outputs:

| Artifact | Path |
|----------|------|
| Portable folder | `dist/TLC-LINK-Portable/` |
| Portable zip | `dist/TLC-LINK-Portable.zip` |
| NSIS (if enabled) | `src-tauri/target/release/bundle/nsis/` |
| MSI (if enabled) | `src-tauri/target/release/bundle/msi/` |

## 4. Portable folder must contain

```
TLC-LINK-Portable/
  TLC-LINK.exe
  binaries/sing-box/sing-box.exe
  binaries/xray/xray.exe
  README.txt
  KNOWN-LIMITATIONS.txt   (copied from docs)
  CHANGELOG-USER.txt      (optional but recommended)
```

## 5. Minimum smoke tests (real Windows)

Copy from `docs/RELEASE-CHECKLIST.md` and mark only what you actually run:

**Must PASS for RC publish:**

1. App starts without cores → clear error (then retest with cores present)
2. Import one VLESS/Trojan/SS link → convert OK
3. System Proxy + sing-box connect → browser works
4. Disconnect → proxy restored
5. System Proxy + xray connect → disconnect restores
6. Emergency Network Reset with dirty proxy state
7. (If Admin) TUN connect + Disconnect cleans TLC-TUN
8. (If Admin) Kill Switch armed → kill core process → internet stays blocked → Reset restores
9. Subscription URL refresh adds nodes
10. Traffic meters move under load (sing-box)

**May remain known-limited in RC:**

- KS service auto-start after reboot
- Sleep/resume matrix
- Code signature

## 6. Checksums & notes

```powershell
powershell -ExecutionPolicy Bypass -File scripts/make-checksums.ps1
```

Publish together:

- `TLC-LINK-Portable-0.2.0-rc.1.zip`
- `SHA256SUMS`
- `docs/KNOWN-LIMITATIONS.md`
- short `RELEASE-NOTES-0.2.0-rc.1.md` (what changed + how to recover network)

## 7. Do not publish if

- Disconnect leaves system proxy stuck
- Emergency Reset fails
- Any UI path shows “connected” without process/tunnel evidence
- Cores are unofficial renames

## Session improvements already in tree (for RC notes)

- KS hostname resolution + fail-closed
- Multi-upstream DNS Guard
- KS startup reconcile + corrupt journal sweep
- Live traffic meters (sing-box)
- Subscription manual refresh
- IPv6-safe TCP latency parse
- VLESS `packetEncoding` allow/forward
