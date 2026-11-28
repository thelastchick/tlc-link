//! Network-mode configuration layer. This module only prepares Core configuration;
//! it does not claim that Windows routes or proxy settings are changed unless the
//! corresponding platform integration is active.
use serde_json::{json, Value};

/// sing-box `mixed` inbound (HTTP + SOCKS) and Xray SOCKS inbound.
pub const SOCKS_PORT: u16 = 2080;
/// Xray has no mixed inbound, so a separate HTTP inbound is used for System Proxy.
pub const XRAY_HTTP_PORT: u16 = 2081;

/// Address Windows System Proxy must point to for the given core, and the port a
/// connectivity probe can speak HTTP-proxy to.
pub fn http_proxy_port(core: &str) -> u16 {
    if core.eq_ignore_ascii_case("xray") { XRAY_HTTP_PORT } else { SOCKS_PORT }
}
pub fn system_proxy_server(core: &str) -> String { format!("127.0.0.1:{}", http_proxy_port(core)) }

/// Every local port the generated configuration listens on in System Proxy mode.
/// All of them must be free BEFORE the core is started (otherwise the probe could end up
/// talking to a foreign process that already owns the port).
pub fn required_ports(core: &str) -> Vec<u16> {
    if core.eq_ignore_ascii_case("xray") { vec![SOCKS_PORT, XRAY_HTTP_PORT] } else { vec![SOCKS_PORT] }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkMode { SystemProxy, Tun }

impl NetworkMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "system-proxy" | "proxy" => Ok(Self::SystemProxy),
            "tun" => Ok(Self::Tun),
            _ => Err("حالت شبکه نامعتبر است؛ system-proxy یا tun را انتخاب کنید".into()),
        }
    }
    pub fn name(self) -> &'static str { match self { Self::SystemProxy => "system-proxy", Self::Tun => "tun" } }
}

/// Adds a real local inbound to the selected Core configuration. TUN currently
/// uses sing-box's native TUN inbound; Xray TUN is rejected rather than faked.
pub fn apply_mode(config: &mut Value, core: &str, mode: NetworkMode, dns_guard: bool) -> Result<(), String> {
    if !config.is_object() { return Err("ریشه کانفیگ باید JSON object باشد".into()); }
    if config.get("inbounds").is_none() { config["inbounds"] = json!([]); }
    let inbounds = config.get_mut("inbounds").and_then(Value::as_array_mut)
        .ok_or("فیلد inbounds باید آرایه باشد")?;
    // Do not duplicate TLC-managed inbounds when a profile is revalidated.
    inbounds.retain(|item| !item.get("tag").and_then(Value::as_str).map(|t| t.starts_with("tlc-inbound")).unwrap_or(false));
    match mode {
        NetworkMode::SystemProxy => {
            if dns_guard {
                return Err("DNS Leak Guard به System TUN نیاز دارد؛ ابتدا System TUN Mode را فعال کنید.".into());
            }
            match core.to_ascii_lowercase().as_str() {
                "sing-box" | "singbox" => inbounds.push(json!({
                    "type":"mixed", "tag":"tlc-inbound", "listen":"127.0.0.1", "listen_port":SOCKS_PORT
                })),
                "xray" => {
                    inbounds.push(json!({
                        "tag":"tlc-inbound", "listen":"127.0.0.1", "port":SOCKS_PORT,
                        "protocol":"socks", "settings":{"auth":"noauth","udp":true},
                        "sniffing":{"enabled":true,"destOverride":["http","tls"]}
                    }));
                    // Windows System Proxy (WinINet) expects an HTTP proxy.
                    inbounds.push(json!({
                        "tag":"tlc-inbound-http", "listen":"127.0.0.1", "port":XRAY_HTTP_PORT,
                        "protocol":"http", "settings":{},
                        "sniffing":{"enabled":true,"destOverride":["http","tls"]}
                    }));
                }
                _ => return Err("Core ناشناخته است".into()),
            }
        }
        NetworkMode::Tun => {
            if core.eq_ignore_ascii_case("xray") {
                return Err("TUN داخلی در این نسخه فقط با sing-box پشتیبانی می‌شود. برای Xray ابتدا System Proxy را انتخاب کنید.".into());
            }
            if !core.eq_ignore_ascii_case("sing-box") && !core.eq_ignore_ascii_case("singbox") {
                return Err("TUN فقط برای sing-box در دسترس است".into());
            }
            inbounds.push(json!({
                "type":"tun",
                "tag":"tlc-inbound",
                "interface_name":"TLC-TUN",
                "address":["172.19.0.1/30", "fdfe:dcba:9876::1/126"],
                "dns_address":["172.19.0.2", "fdfe:dcba:9876::2"],
                "mtu":9000,
                "auto_route":true,
                // On Windows strict_route also prevents ordinary multi-homed DNS leaks.
                "strict_route":dns_guard,
                // Use sing-box DNS interception in TUN mode. DNS Leak Guard additionally
                // enables strict_route to block Windows multi-homed DNS leakage.
                // Protocol sniffing is configured below through a modern route action;
                // the legacy inbound `sniff` field was removed in sing-box 1.13.
                "dns_mode":"hijack"
            }));
            if config.get("route").and_then(Value::as_object).is_none() {
                config["route"] = json!({});
            }
            let proxy_tag = proxy_outbound_tag(config)?;
            let final_tag_is_valid = config.get("route").and_then(|r| r.get("final")).and_then(Value::as_str).map(|tag| {
                config.get("outbounds").and_then(Value::as_array).map(|items| {
                    items.iter().any(|item| item.get("tag").and_then(Value::as_str) == Some(tag))
                }).unwrap_or(false)
            }).unwrap_or(false);
            let route = config.get_mut("route").and_then(Value::as_object_mut)
                .ok_or("فیلد route باید JSON object باشد")?;
            route.insert("auto_detect_interface".into(), json!(true));
            if !final_tag_is_valid {
                route.insert("final".into(), json!(proxy_tag));
            }
            // sing-box 1.11+ replaces inbound sniff fields with route actions.
            // Keep existing profile rules, but run TLC's sniff action first so later
            // routing rules can use the detected protocol/domain.
            let rules = route.entry("rules").or_insert_with(|| json!([]))
                .as_array_mut()
                .ok_or("فیلد route.rules باید آرایه باشد")?;
            rules.retain(|rule| {
                !(rule.get("inbound").and_then(Value::as_str) == Some("tlc-inbound")
                    && rule.get("action").and_then(Value::as_str) == Some("sniff"))
            });
            rules.insert(0, json!({"inbound":"tlc-inbound", "action":"sniff"}));
            // A TUN inbound needs a valid DNS upstream. DNS Leak Guard additionally
            // enables strict Windows DNS routing via strict_route above.
            apply_tun_dns(config, dns_guard, &proxy_tag)?;
        }
    }
    // Local Clash API for traffic stats (sing-box only). Bound to loopback.
    if core.eq_ignore_ascii_case("sing-box") || core.eq_ignore_ascii_case("singbox") {
        inject_clash_api(config);
    }
    Ok(())
}

/// Local-only Clash API controller used for live upload/download counters.
pub const CLASH_API_HOST: &str = "127.0.0.1";
pub const CLASH_API_PORT: u16 = 16789;
pub const CLASH_API_SECRET: &str = "tlc-link-local";

fn inject_clash_api(config: &mut Value) {
    if config.get("experimental").and_then(Value::as_object).is_none() {
        config["experimental"] = json!({});
    }
    if let Some(exp) = config.get_mut("experimental").and_then(Value::as_object_mut) {
        exp.insert(
            "clash_api".into(),
            json!({
                "external_controller": format!("{CLASH_API_HOST}:{CLASH_API_PORT}"),
                "secret": CLASH_API_SECRET,
                "default_mode": "rule"
            }),
        );
    }
}

/// Configures encrypted DNS for TUN mode.
///
/// Design (aligned with mature sing-box / Clash-style clients):
/// 1. **Bootstrap** — plain UDP to several well-known public resolvers by *literal IP*
///    and `detour: direct`. This breaks the chicken-and-egg problem of resolving the
///    proxy hostname and of resolving DoH hostnames themselves.
/// 2. **Protected DoH** — several independent HTTPS upstreams (Cloudflare, Google, Quad9)
///    that go out through the selected proxy outbound. Hostname-based DoH servers
///    use the bootstrap tag as `domain_resolver`.
/// 3. **final** points at the primary DoH tag; strategy prefers IPv4 for predictability.
/// 4. When DNS Leak Guard is on, any pre-existing non-empty `dns.rules` is rejected
///    rather than silently overridden (product honesty).
/// 5. Outbounds whose `server` is a hostname get `domain_resolver = tlc-link-bootstrap`
///    so the first dial does not recurse through the tunnel.
fn apply_tun_dns(config: &mut Value, dns_guard: bool, proxy_tag: &str) -> Result<(), String> {
    // Ensure a usable `direct` outbound exists for bootstrap / local dials.
    ensure_direct_outbound(config);

    if config.get("dns").is_none() {
        config["dns"] = json!({});
    } else if config.get("dns").and_then(Value::as_object).is_none() {
        return Err("کانفیگ DNS موجود ساختار جدید JSON object ندارد؛ برای جلوگیری از حذف تنظیمات سفارشی، DNS Leak Guard اعمال نشد.".into());
    }
    let dns = config
        .get_mut("dns")
        .and_then(Value::as_object_mut)
        .ok_or("فیلد dns باید JSON object باشد")?;

    if dns_guard {
        if let Some(rules) = dns.get("rules") {
            let rules = rules.as_array().ok_or(
                "فیلد dns.rules باید آرایه باشد؛ DNS Leak Guard برای این کانفیگ فعال نشد.",
            )?;
            if !rules.is_empty() {
                return Err(
                    "این کانفیگ قوانین DNS سفارشی دارد. برای جلوگیری از مسیرهای DNS پیش‌بینی‌نشده، ابتدا قوانین DNS سفارشی را از کانفیگ حذف کنید یا DNS Leak Guard را خاموش نگه دارید.".into(),
                );
            }
        }
    }

    if dns.get("servers").is_none() {
        dns.insert("servers".into(), json!([]));
    }
    let servers = dns
        .get_mut("servers")
        .and_then(Value::as_array_mut)
        .ok_or("فیلد dns.servers باید آرایه باشد")?;

    // Remove any previous TLC-managed servers so re-apply is idempotent.
    servers.retain(|server| {
        !matches!(
            server.get("tag").and_then(Value::as_str),
            Some(
                "tlc-link-bootstrap"
                    | "tlc-link-bootstrap-cf"
                    | "tlc-link-bootstrap-google"
                    | "tlc-link-doh"
                    | "tlc-link-doh-cf"
                    | "tlc-link-doh-google"
                    | "tlc-link-doh-quad9"
            )
        )
    });

    // --- Bootstrap: UDP to literal IPs, always direct ---
    // Multiple addresses reduce the chance that a single blocked resolver breaks connect.
    servers.push(json!({
        "type": "udp",
        "tag": "tlc-link-bootstrap",
        "server": "1.1.1.1",
        "server_port": 53,
        "detour": "direct"
    }));
    servers.push(json!({
        "type": "udp",
        "tag": "tlc-link-bootstrap-cf",
        "server": "1.0.0.1",
        "server_port": 53,
        "detour": "direct"
    }));
    servers.push(json!({
        "type": "udp",
        "tag": "tlc-link-bootstrap-google",
        "server": "8.8.8.8",
        "server_port": 53,
        "detour": "direct"
    }));

    // --- Protected DoH upstreams (through the proxy) ---
    // Primary: Cloudflare by IP (no extra resolve needed for the DoH server itself).
    servers.push(json!({
        "type": "https",
        "tag": "tlc-link-doh",
        "server": "1.1.1.1",
        "server_port": 443,
        "path": "/dns-query",
        "tls": { "enabled": true, "server_name": "cloudflare-dns.com" },
        "detour": proxy_tag
    }));
    // Fallback: Google DoH (hostname → resolved via bootstrap).
    servers.push(json!({
        "type": "https",
        "tag": "tlc-link-doh-google",
        "server": "dns.google",
        "server_port": 443,
        "path": "/dns-query",
        "tls": { "enabled": true, "server_name": "dns.google" },
        "domain_resolver": "tlc-link-bootstrap",
        "detour": proxy_tag
    }));
    // Fallback: Quad9 DoH.
    servers.push(json!({
        "type": "https",
        "tag": "tlc-link-doh-quad9",
        "server": "dns.quad9.net",
        "server_port": 443,
        "path": "/dns-query",
        "tls": { "enabled": true, "server_name": "dns.quad9.net" },
        "domain_resolver": "tlc-link-bootstrap",
        "detour": proxy_tag
    }));

    dns.insert("final".into(), json!("tlc-link-doh"));
    dns.insert("strategy".into(), json!("prefer_ipv4"));

    // Resolve proxy endpoint hostnames through the fixed-IP bootstrap resolver.
    // If the selected outbound is a selector/urltest, also cover its members.
    let mut resolver_tags = vec![proxy_tag.to_string()];
    if let Some(outbounds) = config.get("outbounds").and_then(Value::as_array) {
        if let Some(selected) = outbounds
            .iter()
            .find(|o| o.get("tag").and_then(Value::as_str) == Some(proxy_tag))
        {
            if let Some(members) = selected.get("outbounds").and_then(Value::as_array) {
                resolver_tags.extend(members.iter().filter_map(Value::as_str).map(str::to_string));
            }
        }
    }
    if let Some(outbounds) = config.get_mut("outbounds").and_then(Value::as_array_mut) {
        for outbound in outbounds {
            let is_selected = outbound
                .get("tag")
                .and_then(Value::as_str)
                .map(|tag| resolver_tags.iter().any(|candidate| candidate == tag))
                .unwrap_or(false);
            if is_selected {
                let is_domain = outbound
                    .get("server")
                    .and_then(Value::as_str)
                    .map(|server| server.parse::<std::net::IpAddr>().is_err())
                    .unwrap_or(false);
                if is_domain {
                    outbound["domain_resolver"] = json!("tlc-link-bootstrap");
                }
            }
        }
    }
    Ok(())
}

/// Make sure a `direct` outbound exists so bootstrap DNS and local dials have a target.
fn ensure_direct_outbound(config: &mut Value) {
    if config.get("outbounds").is_none() {
        config["outbounds"] = json!([]);
    }
    let Some(outbounds) = config.get_mut("outbounds").and_then(Value::as_array_mut) else {
        return;
    };
    let has_direct = outbounds.iter().any(|o| {
        o.get("tag").and_then(Value::as_str) == Some("direct")
            || o.get("type").and_then(Value::as_str) == Some("direct")
    });
    if !has_direct {
        outbounds.push(json!({ "type": "direct", "tag": "direct" }));
    }
}

/// Prefer the configured route.final outbound if it is a proxy-capable tag;
/// otherwise select the first non-direct/non-block outbound. This also supports
/// imported sing-box profiles whose proxy tag is not literally named `proxy`.
fn proxy_outbound_tag(config: &Value) -> Result<String, String> {
    let outbounds = config.get("outbounds").and_then(Value::as_array)
        .ok_or("برای System TUN باید کانفیگ sing-box حداقل یک outbound داشته باشد.")?;
    let is_proxy_tag = |tag: &str| {
        outbounds.iter().any(|outbound| {
            outbound.get("tag").and_then(Value::as_str) == Some(tag)
                && !matches!(outbound.get("type").and_then(Value::as_str), Some("direct" | "block" | "dns"))
                && !matches!(tag, "direct" | "block")
        })
    };
    if let Some(tag) = config.get("route").and_then(|r| r.get("final")).and_then(Value::as_str) {
        if is_proxy_tag(tag) { return Ok(tag.to_string()); }
    }
    outbounds.iter().find_map(|outbound| {
        let tag = outbound.get("tag").and_then(Value::as_str)?;
        let kind = outbound.get("type").and_then(Value::as_str);
        if !matches!(kind, Some("direct" | "block" | "dns")) && !matches!(tag, "direct" | "block") {
            Some(tag.to_string())
        } else { None }
    }).ok_or_else(|| "هیچ outbound پروکسی معتبری برای TUN و DNS رمزگذاری‌شده در کانفیگ پیدا نشد.".into())
}


/// Applies the selected routing mode (`rule`, `global`, `direct`) to the core config.
pub fn apply_routing_mode(config: &mut Value, core: &str, routing_mode: &str) {
    let mode = routing_mode.trim().to_ascii_lowercase();
    let proxy_tag = proxy_outbound_tag(config).unwrap_or_else(|_| "proxy".to_string());
    let target = if mode == "direct" { "direct".to_string() } else { proxy_tag };
    match core.to_ascii_lowercase().as_str() {
        "sing-box" | "singbox" => {
            if config.get("route").is_none() {
                config["route"] = json!({});
            }
            if let Some(route) = config.get_mut("route").and_then(Value::as_object_mut) {
                route.insert("final".into(), json!(target));
            }
        }
        "xray" => {
            if mode == "direct" {
                config["routing"] = json!({
                    "domainStrategy": "AsIs",
                    "rules": [{ "type": "field", "network": "tcp,udp", "outboundTag": "direct" }]
                });
            } else if mode == "global" {
                config["routing"] = json!({
                    "domainStrategy": "AsIs",
                    "rules": [{ "type": "field", "network": "tcp,udp", "outboundTag": "proxy" }]
                });
            }
        }
        _ => {}
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn required_ports_cover_every_local_inbound() {
        assert_eq!(required_ports("sing-box"), vec![2080]);
        assert_eq!(required_ports("xray"), vec![2080, 2081]);
    }
    #[test] fn system_proxy_adds_local_mixed_inbound() {
        let mut c=json!({"inbounds":[],"outbounds":[],"route":{"final":"proxy"}});
        apply_mode(&mut c,"sing-box",NetworkMode::SystemProxy,false).unwrap();
        assert_eq!(c["inbounds"][0]["listen_port"],2080);
    }
    #[test] fn tun_rejects_xray_instead_of_claiming_support() {
        let mut c=json!({"inbounds":[],"outbounds":[],"route":{"final":"proxy"}});
        assert!(apply_mode(&mut c,"xray",NetworkMode::Tun,false).is_err());
    }
    #[test] fn tun_adds_singbox_tun_inbound() {
        // TUN requires a real proxy outbound; an empty outbound list is an
        // invalid fixture and should be rejected by proxy_outbound_tag().
        let mut c=json!({
            "inbounds":[],
            "outbounds":[{"type":"vless","tag":"proxy","server":"node.example","server_port":443}],
            "route":{"final":"proxy"}
        });
        apply_mode(&mut c,"sing-box",NetworkMode::Tun,false).unwrap();
        assert_eq!(c["inbounds"][0]["type"],"tun");
        assert_eq!(c["route"]["auto_detect_interface"],true);
        assert_eq!(c["inbounds"][0]["dns_mode"],"hijack");
        assert_eq!(c["inbounds"][0]["dns_address"][0],"172.19.0.2");
        assert!(c["dns"]["servers"].as_array().is_some());
        assert_eq!(c["inbounds"][0]["strict_route"],false);
        assert!(c["inbounds"][0].get("sniff").is_none());
        assert_eq!(c["route"]["rules"][0], json!({"inbound":"tlc-inbound", "action":"sniff"}));
    }

    #[test] fn dns_guard_requires_tun() {
        let mut c=json!({"inbounds":[],"outbounds":[]});
        assert!(apply_mode(&mut c,"sing-box",NetworkMode::SystemProxy,true).is_err());
    }
    #[test] fn tun_dns_guard_uses_modern_doh_and_ipv6() {
        let mut c=json!({"inbounds":[],"outbounds":[{"type":"vless","tag":"proxy","server":"node.example","server_port":443}],"route":{}});
        apply_mode(&mut c,"sing-box",NetworkMode::Tun,true).unwrap();
        assert_eq!(c["inbounds"][0]["dns_mode"],"hijack");
        assert_eq!(c["inbounds"][0]["strict_route"],true);
        // Bootstrap is UDP; primary DoH is HTTPS tagged tlc-link-doh.
        let servers = c["dns"]["servers"].as_array().expect("dns.servers");
        assert!(servers.iter().any(|s| s["tag"] == "tlc-link-bootstrap" && s["type"] == "udp"));
        assert!(servers.iter().any(|s| s["tag"] == "tlc-link-doh" && s["type"] == "https"));
        assert!(servers.iter().any(|s| s["tag"] == "tlc-link-doh-google" && s["type"] == "https"));
        assert!(servers.iter().any(|s| s["tag"] == "tlc-link-doh-quad9" && s["type"] == "https"));
        assert_eq!(c["dns"]["final"], "tlc-link-doh");
        assert_eq!(c["dns"]["strategy"], "prefer_ipv4");
        assert_eq!(c["outbounds"][0]["domain_resolver"],"tlc-link-bootstrap");
        // direct outbound must exist for bootstrap detour
        assert!(c["outbounds"].as_array().unwrap().iter().any(|o| o["tag"] == "direct"));
    }
    #[test] fn tun_supports_non_default_proxy_outbound_tag() {
        let mut c=json!({"inbounds":[],"outbounds":[{"type":"vless","tag":"edge-eu","server":"node.example","server_port":443},{"type":"direct","tag":"direct"}],"route":{"final":"edge-eu"}});
        apply_mode(&mut c,"sing-box",NetworkMode::Tun,true).unwrap();
        let servers = c["dns"]["servers"].as_array().expect("dns.servers");
        let primary = servers.iter().find(|s| s["tag"] == "tlc-link-doh").expect("primary doh");
        assert_eq!(primary["detour"], "edge-eu");
        assert_eq!(c["outbounds"][0]["domain_resolver"],"tlc-link-bootstrap");
    }
    #[test] fn global_routing_uses_custom_proxy_tag() {
        let mut c=json!({"inbounds":[],"outbounds":[{"type":"vless","tag":"edge-eu","server":"node.example","server_port":443},{"type":"direct","tag":"direct"}],"route":{"final":"edge-eu"}});
        apply_routing_mode(&mut c,"sing-box","global");
        assert_eq!(c["route"]["final"],"edge-eu");
    }
    #[test] fn tun_repairs_invalid_final_tag_using_a_real_proxy_outbound() {
        let mut c=json!({"inbounds":[],"outbounds":[{"type":"vless","tag":"edge-eu","server":"node.example","server_port":443}],"route":{"final":"proxy"}});
        apply_mode(&mut c,"sing-box",NetworkMode::Tun,false).unwrap();
        assert_eq!(c["route"]["final"],"edge-eu");
        let servers = c["dns"]["servers"].as_array().expect("dns.servers");
        let primary = servers.iter().find(|s| s["tag"] == "tlc-link-doh").expect("primary doh");
        assert_eq!(primary["detour"], "edge-eu");
    }
    #[test] fn dns_guard_rejects_custom_dns_rules_instead_of_claiming_protection() {
        let mut c=json!({"inbounds":[],"outbounds":[],"route":{},"dns":{"rules":[{"domain":["example.com"],"server":"custom"}]}});
        assert!(apply_mode(&mut c,"sing-box",NetworkMode::Tun,true).is_err());
    }
        #[test] fn clash_api_is_injected_for_singbox() {
        let mut c=json!({"inbounds":[],"outbounds":[{"type":"vless","tag":"proxy","server":"1.2.3.4","server_port":443}],"route":{"final":"proxy"}});
        apply_mode(&mut c,"sing-box",NetworkMode::SystemProxy,false).unwrap();
        assert_eq!(c["experimental"]["clash_api"]["external_controller"], format!("{CLASH_API_HOST}:{CLASH_API_PORT}"));
        assert_eq!(c["experimental"]["clash_api"]["secret"], CLASH_API_SECRET);
    }
#[test] fn xray_gets_socks_and_http_inbounds_without_duplicates() {
        let mut c=json!({"inbounds":[],"outbounds":[]});
        apply_mode(&mut c,"xray",NetworkMode::SystemProxy,false).unwrap();
        apply_mode(&mut c,"xray",NetworkMode::SystemProxy,false).unwrap();
        assert_eq!(c["inbounds"].as_array().unwrap().len(),2);
        assert_eq!(system_proxy_server("xray"),"127.0.0.1:2081");
        assert_eq!(system_proxy_server("sing-box"),"127.0.0.1:2080");
    }
}
