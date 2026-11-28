# Pass 7 implementation status

| Area | Implemented in source | Verified at runtime |
|---|---:|---:|
| Local sing-box mixed inbound | Yes | No (Core binaries unavailable) |
| Local Xray SOCKS inbound | Yes | No (Core binaries unavailable) |
| sing-box TUN config generation | Yes | No (Windows adapter/privilege test required) |
| Xray TUN guard | Yes | Static code only |
| UI passes selected mode to Rust | Yes | JavaScript syntax checked |
| Windows Internet Settings apply/restore | No | No |
| Crash-safe proxy restoration | No | No |
| Real traffic probe and counters | No | No |
| Enforced Kill Switch / DNS leak protection | No | No |
| Offline QR dependencies | No (CDN allowed by CSP) | No |

This source package is a Pass 7 engineering preview, not a production-ready VPN release. Do not advertise the two modes as fully operational until the Windows integration and runtime acceptance tests are complete.
