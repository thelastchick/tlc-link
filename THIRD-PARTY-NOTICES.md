# Third-party notices (draft — verify before distribution)

| Component | Use | License (verify upstream) | Version pinned |
|---|---|---|---|
| Tauri 2 | App shell | MIT / Apache-2.0 | via Cargo.lock (not generated yet) |
| windows-sys 0.59 | Registry, Job Objects, process identity | MIT / Apache-2.0 | 0.59 |
| serde, serde_json, thiserror | Rust libraries | MIT / Apache-2.0 | via Cargo.lock |
| sing-box | Proxy core (external executable) | GPL-3.0-or-later with upstream name-use clause | **NOT PINNED — no binary obtained** |
| Xray-core | Proxy core (external executable) | MPL-2.0 | **NOT PINNED — no binary obtained** |
| qrcode (npm `qrcode@1.5.4`) | QR generation | MIT | fetched by `scripts/fetch-vendor.ps1` |
| jsQR (`jsqr@1.4.0`) | QR decoding | Apache-2.0 | fetched by `scripts/fetch-vendor.ps1` |

Cores run as separate processes; ship their license texts next to the executables.
No code from v2rayN was copied.
