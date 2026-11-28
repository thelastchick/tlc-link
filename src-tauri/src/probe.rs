//! Real end-to-end connectivity probe. Requests are sent THROUGH the local HTTP proxy
//! inbound of the running core and timed. A started process is not proof of connectivity:
//! (1) a plain-HTTP 204 and (2) an HTTPS `CONNECT` tunnel to port 443 both have to succeed.
//! Failures are classified (port closed / timeout / refused / upstream error / bad response) so
//! the user gets an actionable message. A TLS handshake is not performed here (no TLS crate);
//! the CONNECT stage proves the tunnel reaches the remote host on :443.
use serde::Serialize;
use std::{
    io::{ErrorKind, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    time::{Duration, Instant},
};

const TARGET_HOST: &str = "www.gstatic.com";
const TARGET_PATH: &str = "/generate_204";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeFailure { PortClosed, Timeout, ConnectionReset, UpstreamError, BadResponse }

#[derive(Clone, Debug, Serialize)]
pub struct ProbeResult {
    pub ok: bool,
    pub latency_ms: Option<u64>,
    pub failure: Option<ProbeFailure>,
    pub error: Option<String>,
}

/// True when 127.0.0.1:`port` can be bound right now (nobody else owns it).
pub fn port_is_free(port: u16) -> bool { TcpListener::bind(("127.0.0.1", port)).is_ok() }

/// Waits until the core's local inbound accepts TCP connections.
pub fn wait_for_port(port: u16, total: Duration) -> bool {
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    let deadline = Instant::now() + total;
    while Instant::now() < deadline {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok() { return true; }
        std::thread::sleep(Duration::from_millis(150));
    }
    false
}

/// `port` must be an HTTP-capable local proxy port (sing-box mixed, Xray http inbound).
pub fn through_http_proxy(port: u16, timeout: Duration) -> ProbeResult {
    let started = Instant::now();
    let stage1 = http_204(port, timeout);
    let latency = started.elapsed().as_millis() as u64;
    if let Err((failure, error)) = stage1 {
        return ProbeResult { ok: false, latency_ms: None, failure: Some(failure), error: Some(format!("تست HTTP: {error}")) };
    }
    if let Err((failure, error)) = connect_443(port, timeout) {
        return ProbeResult { ok: false, latency_ms: None, failure: Some(failure), error: Some(format!("تست تونل HTTPS (CONNECT): {error}")) };
    }
    ProbeResult { ok: true, latency_ms: Some(latency), failure: None, error: None }
}

type Fail = (ProbeFailure, String);

fn classify(e: &std::io::Error, what: &str) -> Fail {
    match e.kind() {
        ErrorKind::TimedOut | ErrorKind::WouldBlock => (ProbeFailure::Timeout, format!("{what}: پاسخی در مهلت مقرر نرسید (سرور کند، مسدود یا غیرقابل دسترس است)")),
        ErrorKind::ConnectionRefused => (ProbeFailure::PortClosed, format!("{what}: اتصال به پروکسی محلی رد شد")),
        ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted | ErrorKind::BrokenPipe | ErrorKind::UnexpectedEof => (ProbeFailure::ConnectionReset, format!("{what}: اتصال توسط هسته یا سرور قطع شد")),
        _ => (ProbeFailure::ConnectionReset, format!("{what}: {e}")),
    }
}

fn open(port: u16, timeout: Duration) -> Result<TcpStream, Fail> {
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    let s = TcpStream::connect_timeout(&addr, timeout).map_err(|e| classify(&e, &format!("اتصال به پروکسی محلی (پورت {port})")))?;
    let _ = s.set_read_timeout(Some(timeout));
    let _ = s.set_write_timeout(Some(timeout));
    Ok(s)
}

fn read_status_line(stream: &mut TcpStream) -> Result<String, Fail> {
    let mut buf = [0u8; 256];
    let mut got = Vec::new();
    while !got.windows(2).any(|w| w == b"\r\n") && got.len() < 1024 {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => got.extend_from_slice(&buf[..n]),
            Err(e) => return Err(classify(&e, "دریافت پاسخ")),
        }
    }
    if got.is_empty() { return Err((ProbeFailure::ConnectionReset, "پاسخی دریافت نشد (اتصال بدون داده بسته شد)".into())); }
    Ok(String::from_utf8_lossy(&got).lines().next().unwrap_or("").trim().to_string())
}

fn status_code(line: &str) -> Option<u16> {
    let mut it = line.split_whitespace();
    if !it.next()?.starts_with("HTTP/") { return None; }
    it.next()?.parse().ok()
}

fn http_204(port: u16, timeout: Duration) -> Result<(), Fail> {
    let mut s = open(port, timeout)?;
    let req = format!("GET http://{TARGET_HOST}{TARGET_PATH} HTTP/1.1\r\nHost: {TARGET_HOST}\r\nConnection: close\r\nUser-Agent: TLC-LINK-probe\r\n\r\n");
    s.write_all(req.as_bytes()).map_err(|e| classify(&e, "ارسال درخواست"))?;
    let line = read_status_line(&mut s)?;
    match status_code(&line) {
        Some(204) => Ok(()),
        Some(c) if (500..600).contains(&c) => Err((ProbeFailure::UpstreamError, format!("هسته خطای {c} برگرداند؛ معمولاً یعنی سرور VPN در دسترس نیست یا DNS/اتصال خروجی شکست خورده است"))),
        _ => Err((ProbeFailure::BadResponse, format!("پاسخ نامعتبر یا غیرمنتظره: {line}"))),
    }
}

fn connect_443(port: u16, timeout: Duration) -> Result<(), Fail> {
    let mut s = open(port, timeout)?;
    let req = format!("CONNECT {TARGET_HOST}:443 HTTP/1.1\r\nHost: {TARGET_HOST}:443\r\nUser-Agent: TLC-LINK-probe\r\n\r\n");
    s.write_all(req.as_bytes()).map_err(|e| classify(&e, "ارسال CONNECT"))?;
    let line = read_status_line(&mut s)?;
    match status_code(&line) {
        Some(200) => Ok(()),
        Some(c) if (500..600).contains(&c) => Err((ProbeFailure::UpstreamError, format!("تونل HTTPS با خطای {c} رد شد (سرور، DNS یا TLS/Reality سمت سرور مشکل دارد)"))),
        _ => Err((ProbeFailure::BadResponse, format!("پاسخ نامعتبر به CONNECT: {line}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::{Read, Write}, net::TcpListener, thread};

    /// Fake proxy that answers each accepted connection with the next canned reply.
    fn fake_proxy(replies: Vec<&'static str>) -> u16 {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        thread::spawn(move || {
            for reply in replies {
                if let Ok((mut s, _)) = l.accept() {
                    let mut b = [0u8; 512];
                    let _ = s.read(&mut b);
                    let _ = s.write_all(reply.as_bytes());
                }
            }
        });
        port
    }

    #[test]
    fn http_and_connect_success_gives_latency() {
        let port = fake_proxy(vec!["HTTP/1.1 204 No Content\r\n\r\n", "HTTP/1.1 200 Connection established\r\n\r\n"]);
        let r = through_http_proxy(port, Duration::from_secs(2));
        assert!(r.ok && r.latency_ms.is_some() && r.failure.is_none());
    }

    #[test]
    fn upstream_5xx_is_classified() {
        let port = fake_proxy(vec!["HTTP/1.1 502 Bad Gateway\r\n\r\n"]);
        let r = through_http_proxy(port, Duration::from_secs(2));
        assert!(!r.ok && r.failure == Some(ProbeFailure::UpstreamError) && r.latency_ms.is_none());
    }

    #[test]
    fn garbage_is_bad_response() {
        let port = fake_proxy(vec!["NOT HTTP AT ALL\r\n\r\n"]);
        assert_eq!(through_http_proxy(port, Duration::from_secs(2)).failure, Some(ProbeFailure::BadResponse));
    }

    #[test]
    fn http_ok_but_connect_blocked_is_failure() {
        let port = fake_proxy(vec!["HTTP/1.1 204 No Content\r\n\r\n", "HTTP/1.1 503 Service Unavailable\r\n\r\n"]);
        let r = through_http_proxy(port, Duration::from_secs(2));
        assert!(!r.ok && r.error.unwrap().contains("CONNECT"));
    }

    #[test]
    fn silent_proxy_times_out() {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        thread::spawn(move || { if let Ok((s, _)) = l.accept() { thread::sleep(Duration::from_secs(3)); drop(s); } });
        let r = through_http_proxy(port, Duration::from_millis(400));
        assert_eq!(r.failure, Some(ProbeFailure::Timeout));
    }

    #[test]
    fn closed_port_reports_an_error() {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        drop(l);
        assert!(!through_http_proxy(port, Duration::from_millis(500)).ok);
    }

    #[test]
    fn port_free_detection() {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        assert!(!port_is_free(port));
        drop(l);
        assert!(port_is_free(port));
    }
}


/// Measures TCP handshake latency to a profile's configured server endpoint.
/// This is an endpoint reachability measurement, not an ICMP echo and not a
/// full VPN-throughput test. The UI labels the value as latency in milliseconds.
pub fn endpoint_tcp_latency(address: &str, timeout: Duration) -> Result<u64, String> {
    use std::net::ToSocketAddrs;

    let value = address.trim();
    let (host, port) = parse_host_port(value)?;
    if host.is_empty() || host.contains('/') || host.contains(' ') {
        return Err("آدرس میزبان معتبر نیست".into());
    }
    if port == 0 {
        return Err("پورت سرور معتبر نیست".into());
    }

    let resolved: Vec<SocketAddr> = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|e| format!("خطای DNS یا آدرس سرور: {e}"))?
        .collect();
    if resolved.is_empty() {
        return Err("آدرس سرور به IP تبدیل نشد".into());
    }

    let started = Instant::now();
    let mut last_error = None;
    for socket in resolved {
        let remaining = timeout.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            break;
        }
        match TcpStream::connect_timeout(&socket, remaining) {
            Ok(stream) => {
                drop(stream);
                return Ok(started.elapsed().as_millis() as u64);
            }
            Err(e) => last_error = Some(e.to_string()),
        }
    }
    Err(last_error.unwrap_or_else(|| "مهلت تست اتصال به پایان رسید".into()))
}

fn parse_host_port(value: &str) -> Result<(String, u16), String> {
    let s = value.trim();
    if let Some(rest) = s.strip_prefix('[') {
        let (host, tail) = rest
            .split_once(']')
            .ok_or_else(|| "آدرس IPv6 نامعتبر است".to_string())?;
        let port_text = tail
            .strip_prefix(':')
            .ok_or_else(|| "آدرس سرور باید به شکل [host]:port باشد".to_string())?;
        let port: u16 = port_text
            .trim()
            .parse()
            .map_err(|_| "پورت سرور معتبر نیست".to_string())?;
        return Ok((host.to_string(), port));
    }
    let (host, port_text) = s
        .rsplit_once(':')
        .ok_or_else(|| "آدرس سرور باید به شکل host:port باشد".to_string())?;
    let port: u16 = port_text
        .trim()
        .parse()
        .map_err(|_| "پورت سرور معتبر نیست".to_string())?;
    Ok((host.trim().to_string(), port))
}


#[cfg(test)]
mod endpoint_latency_tests {
    use super::endpoint_tcp_latency;
    use std::{net::TcpListener, thread, time::Duration};

    #[test]
    fn measures_tcp_connection_to_local_endpoint() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let acceptor = thread::spawn(move || {
            let _ = listener.accept();
        });
        let result = endpoint_tcp_latency(
            &format!("127.0.0.1:{port}"),
            Duration::from_secs(2),
        );
        assert!(result.is_ok());
        acceptor.join().unwrap();
    }

    #[test]
    fn rejects_invalid_endpoint() {
        assert!(endpoint_tcp_latency("not-an-endpoint", Duration::from_millis(500)).is_err());
        assert!(endpoint_tcp_latency("example.com:notaport", Duration::from_millis(500)).is_err());

    #[test]
    fn parse_host_port_supports_ipv6_brackets() {
        let (h, p) = super::parse_host_port("[2001:db8::1]:443").unwrap();
        assert_eq!(h, "2001:db8::1");
        assert_eq!(p, 443);
        let (h2, p2) = super::parse_host_port("example.com:8443").unwrap();
        assert_eq!(h2, "example.com");
        assert_eq!(p2, 8443);
    }

    }
}
