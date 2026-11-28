//! Named-pipe protocol between the portable UI and `tlc-ks-service`.
//! Pipe name: \\.\pipe\tlc-ks-svc-v1

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

pub const PIPE_NAME: &str = r"\\.\pipe\tlc-ks-svc-v1";
pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum KsRequest {
    Ping { version: u32 },
    Arm {
        host: String,
        port: u16,
        protocol: String,
        /// Dotted IP or IPv6 literal already resolved by the UI.
        ip: String,
        previous_ip: Option<String>,
    },
    Release,
    EmergencyClear,
    Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum KsResponse {
    Ok { message: String },
    Armed { filter_ids: Vec<u64>, message: String },
    Status {
        state: String,
        armed: bool,
        generation: u64,
        service: bool,
    },
    Err { message: String },
}

impl KsRequest {
    pub fn parse_arm_ips(&self) -> Result<(IpAddr, Option<IpAddr>), String> {
        match self {
            KsRequest::Arm { ip, previous_ip, .. } => {
                let a: IpAddr = ip
                    .parse()
                    .map_err(|_| format!("IP سرور نامعتبر برای Kill Switch: {ip}"))?;
                let b = match previous_ip {
                    Some(p) => Some(
                        p.parse()
                            .map_err(|_| format!("IP قبلی نامعتبر: {p}"))?,
                    ),
                    None => None,
                };
                Ok((a, b))
            }
            _ => Err("not an arm request".into()),
        }
    }
}
