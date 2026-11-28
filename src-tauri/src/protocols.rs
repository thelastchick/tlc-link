//! Basic URI-to-core configuration conversion. This intentionally rejects URI features
//! it cannot faithfully represent rather than silently dropping security settings.
use serde_json::{json, Value};

#[derive(Debug)] struct UriParts { scheme:String, authority:String, path:String, query:String, fragment:String }
fn parse_uri(s:&str)->Result<UriParts,String>{
 let (scheme,rest)=s.trim().split_once("://").ok_or("URI must contain ://")?;
 let scheme=scheme.to_ascii_lowercase(); let (without_frag,fragment)=rest.split_once('#').unwrap_or((rest,""));
 let (authority_and_path,query)=without_frag.split_once('?').unwrap_or((without_frag,""));
 let (authority,path)=if scheme=="ss"||scheme=="vmess" {(authority_and_path,"")} else {authority_and_path.split_once('/').unwrap_or((authority_and_path,""))};
 Ok(UriParts{scheme,authority:authority.to_string(),path:path.to_string(),query:query.to_string(),fragment:decode(fragment)})
}
fn hexval(b:u8)->Option<u8>{match b{b'0'..=b'9'=>Some(b-b'0'),b'a'..=b'f'=>Some(b-b'a'+10),b'A'..=b'F'=>Some(b-b'A'+10),_=>None}}
/// Percent-decoding that works on bytes (never slices a str, so it cannot panic on
/// multi-byte UTF-8). `plus_is_space` is only correct for query strings and
/// fragments; in userinfo (passwords, UUIDs) a literal '+' must be preserved.
fn pct(s:&str,plus_is_space:bool)->String{
 let b=s.as_bytes(); let mut out=Vec::with_capacity(b.len()); let mut i=0;
 while i<b.len(){
  if b[i]==b'%' && i+2<b.len(){ if let (Some(h),Some(l))=(hexval(b[i+1]),hexval(b[i+2])){out.push(h*16+l);i+=3;continue;} }
  if plus_is_space && b[i]==b'+' {out.push(b' ')} else {out.push(b[i])}
  i+=1;
 }
 String::from_utf8_lossy(&out).into_owned()
}
fn decode(s:&str)->String{pct(s,true)}
fn decode_user(s:&str)->String{pct(s,false)}
fn query(q:&str)->std::collections::HashMap<String,String>{q.split('&').filter_map(|x|x.split_once('=').map(|(k,v)|(decode(k),decode(v)))).collect()}
fn split_hostport(s:&str)->Result<(String,u16),String>{let s=s.trim(); if let Some(rest)=s.strip_prefix('['){let (host,tail)=rest.split_once(']').ok_or("Invalid IPv6 host")?;let port=tail.strip_prefix(':').ok_or("Missing port")?.parse().map_err(|_|"Invalid port")?;return Ok((host.into(),port));} let (h,p)=s.rsplit_once(':').ok_or("Server must include port")?;if h.is_empty(){return Err("Server host is empty".into())} Ok((h.into(),p.parse().map_err(|_|"Invalid port")?))}
fn decode_b64(s:&str)->Result<Vec<u8>,String>{
 let chars="ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
 let mut out=Vec::new(); let mut acc:u32=0; let mut bits=0u8; let mut padding=false;
 for c in s.trim().chars(){
  if c=='=' { padding=true; continue; }
  if padding { return Err("Invalid base64 padding".into()); }
  let c=match c {'-'=>'+','_'=>'/',other=>other};
  let v=chars.find(c).ok_or("Invalid base64 character")? as u32;
  acc=(acc<<6)|v; bits+=6;
  if bits>=8 { bits-=8; out.push((acc>>bits) as u8); acc &= (1u32<<bits)-1; }
 }
 if bits==6 || (bits>0 && acc!=0) { return Err("Invalid base64 length or trailing bits".into()); }
 Ok(out)
}
fn auth_host(a:&str)->Result<(String,String,String),String>{let (auth,hostport)=a.rsplit_once('@').ok_or("URI must contain credentials and @host:port")?;let (host,port)=split_hostport(hostport)?;Ok((decode_user(auth),host,port.to_string()))}
fn make_outbound_raw(core:&str,uri:&str)->Result<Value,String>{
 let p=parse_uri(uri)?; let q=query(&p.query);
 match p.scheme.as_str(){
 "vless"=>{let uuid=decode_user(&p.authority.split('@').next().unwrap_or("")); let (h,po)=split_hostport(p.authority.split('@').nth(1).ok_or("VLESS missing server")?)?; let sec=q.get("security").map(String::as_str).unwrap_or("none");let mut v=json!({"type":"vless","tag":"proxy","server":h,"server_port":po,"uuid":uuid,"flow":q.get("flow").cloned().unwrap_or_default()});
  // Optional packet encoding used by many modern clients (xudp / packet).
  if let Some(pe)=q.get("packetEncoding").or_else(||q.get("packet-encoding")).filter(|x|!x.is_empty()){
    v["packet_encoding"]=json!(pe);
  }
  if sec=="tls"||sec=="reality"{v["tls"]=json!({"enabled":true,"server_name":q.get("sni").or(q.get("serverName")).cloned().unwrap_or(h.clone()),"insecure":q.get("allowInsecure").map(|x|x=="1"||x=="true").unwrap_or(false)});if sec=="reality"{v["tls"]["reality"]=json!({"enabled":true,"public_key":q.get("pbk").or(q.get("publicKey")).cloned().unwrap_or_default(),"short_id":q.get("sid").or(q.get("shortId")).cloned().unwrap_or_default()});}} Ok(v)}
 "trojan"=>{let (password,h,po)=auth_host(&p.authority)?;let mut v=json!({"type":"trojan","tag":"proxy","server":h,"server_port":po,"password":password});if q.get("security").map(|x|x=="tls").unwrap_or(true){v["tls"]=json!({"enabled":true,"server_name":q.get("sni").or(q.get("peer")).cloned().unwrap_or_else(||h.clone()),"insecure":q.get("allowInsecure").map(|x|x=="1"||x=="true").unwrap_or(false)});}Ok(v)}
 "ss"=>{
  let payload=p.authority.strip_prefix("//").unwrap_or(&p.authority);
  let (credential,host_port)=if let Some((userinfo,hostport))=payload.rsplit_once('@'){
   let decoded_user=decode_b64(userinfo).ok().and_then(|b|String::from_utf8(b).ok());
   let creds=decoded_user.filter(|x|x.contains(':')).unwrap_or_else(||decode_user(userinfo));
   (creds,hostport.to_string())
  }else{
   let decoded=decode_b64(payload)?; let text=String::from_utf8(decoded).map_err(|_|"Invalid Shadowsocks base64")?;
   let (cred,hp)=text.rsplit_once('@').ok_or("Shadowsocks URI needs method:password@host:port")?;
   (cred.to_string(),hp.to_string())
  };
  let (method,password)=credential.split_once(':').ok_or("Missing Shadowsocks cipher/password")?;
  if method.trim().is_empty()||password.is_empty(){return Err("Shadowsocks method or password is empty".into());}
  let (h,po)=split_hostport(&host_port)?;
  Ok(json!({"type":"shadowsocks","tag":"proxy","server":h,"server_port":po,"method":method,"password":password}))
 }
 "vmess"=>{let decoded=decode_b64(&p.authority)?;let d:Value=serde_json::from_slice(&decoded).map_err(|_|"Invalid VMess JSON")?;let h=d["add"].as_str().ok_or("VMess missing add")?;let po=d["port"].as_str().and_then(|x|x.parse::<u16>().ok()).or(d["port"].as_u64().map(|x|x as u16)).ok_or("VMess invalid port")?;let id=d["id"].as_str().ok_or("VMess missing id")?;let mut v=json!({"type":"vmess","tag":"proxy","server":h,"server_port":po,"uuid":id,"security":d["scy"].as_str().unwrap_or("auto"),"alter_id":d["aid"].as_str().unwrap_or("0").parse::<u32>().unwrap_or(0)});if d["tls"].as_str()==Some("tls"){v["tls"]=json!({"enabled":true,"server_name":d["sni"].as_str().unwrap_or(h),"insecure":d["allowInsecure"].as_str()==Some("1")});}let net=d["net"].as_str().unwrap_or("tcp");if net=="ws"{v["transport"]=json!({"type":"ws","path":d["path"].as_str().unwrap_or(""),"headers":{"Host":d["host"].as_str().unwrap_or("")}});}else if net=="grpc"{v["transport"]=json!({"type":"grpc","service_name":d["path"].as_str().unwrap_or("")});}else if net!="tcp"{return Err(format!("Unsupported VMess network: {net}"));}if net=="tcp"&&!matches!(d["type"].as_str().unwrap_or("none"),"none"|""){return Err("Unsupported VMess TCP header type (http obfuscation)".into());}if let Some(fp)=d["fp"].as_str().filter(|x|!x.is_empty()){if v.get("tls").is_some(){v["tls"]["utls"]=json!({"enabled":true,"fingerprint":fp});}}Ok(v)}
 "hysteria2"|"hy2"=>{let (pass,h,po)=auth_host(&p.authority)?;let mut v=json!({"type":"hysteria2","tag":"proxy","server":h,"server_port":po,"password":pass});v["tls"]=json!({"enabled":true,"server_name":q.get("sni").cloned().unwrap_or_else(||h.clone()),"insecure":q.get("insecure").map(|x|x=="1").unwrap_or(false)});Ok(v)}
 "socks5"|"socks"=>{let hostpart=p.authority.rsplit_once('@').map(|x|x.1).unwrap_or(&p.authority);let (host,port)=split_hostport(hostpart)?;let mut v=json!({"type":"socks","tag":"proxy","server":host,"server_port":port});if let Some(u)=p.authority.split('@').next().filter(|_|p.authority.contains('@')){let (user,pw)=u.split_once(':').unwrap_or((u,""));v["username"]=json!(decode_user(user));v["password"]=json!(decode_user(pw));}Ok(v)}
 _=>Err(format!("Unsupported protocol: {}",p.scheme))
 }
}

type Query=std::collections::HashMap<String,String>;
/// Applies transport / uTLS / ALPN parameters shared by VLESS and Trojan links and
/// rejects parameters TLC LINK cannot represent, instead of silently dropping them.
fn finish_outbound(v:&mut Value,scheme:&str,q:&Query)->Result<(),String>{
 reject_unsupported_params(scheme,q)?;
 if matches!(scheme,"vless"|"trojan"){
  if let Some(e)=q.get("encryption"){ if !e.is_empty()&&e!="none"{return Err(format!("Unsupported encryption parameter: {e}"));} }
  if let Some(h)=q.get("headerType"){ if !h.is_empty()&&h!="none"{return Err(format!("Unsupported TCP headerType: {h}"));} }
  let t=q.get("type").map(String::as_str).unwrap_or("tcp");
  let host=q.get("host").cloned().unwrap_or_default(); let path=q.get("path").cloned().unwrap_or_default();
  match t{
   "tcp"=>{}
   "ws"=>{v["transport"]=json!({"type":"ws","path":path,"headers":{"Host":host}});}
   "grpc"=>{v["transport"]=json!({"type":"grpc","service_name":q.get("serviceName").cloned().unwrap_or_default()});}
   "httpupgrade"=>{v["transport"]=json!({"type":"httpupgrade","host":host,"path":path});}
   "http"|"h2"=>{v["transport"]=json!({"type":"http","host":host.split(',').filter(|x|!x.is_empty()).collect::<Vec<_>>(),"path":path});}
   other=>return Err(format!("Unsupported transport: {other}")),
  }
 }
 if v.get("tls").is_some(){
  if let Some(a)=q.get("alpn"){ let list:Vec<&str>=a.split(',').filter(|x|!x.is_empty()).collect(); if !list.is_empty(){v["tls"]["alpn"]=json!(list);} }
  let reality=v["tls"].get("reality").is_some();
  let fp=q.get("fp").cloned().filter(|x|!x.is_empty()).or_else(||if reality{Some("chrome".to_string())}else{None});
  if let Some(fp)=fp{ v["tls"]["utls"]=json!({"enabled":true,"fingerprint":fp}); }
 }
 Ok(())
}

/// Every query parameter must be either consumed by the converter or explicitly rejected,
/// so a link is never "converted successfully" while part of it was silently dropped.
/// (`spx` is a client-side Reality crawl hint with no effect on the tunnel; it is accepted and
/// intentionally not forwarded.)
fn reject_unsupported_params(scheme:&str,q:&Query)->Result<(),String>{
 let allowed:&[&str]=match scheme{
  "vless"=>&["security","type","flow","sni","serverName","allowInsecure","pbk","publicKey","sid","shortId","spx","fp","alpn","encryption","headerType","host","path","serviceName","mode","packetEncoding","packet-encoding"],
  "trojan"=>&["security","type","sni","peer","allowInsecure","fp","alpn","encryption","headerType","host","path","serviceName","mode"],
  "hysteria2"|"hy2"=>&["sni","insecure"],
  "vmess"=>&["remarks"],
  _=>&[],
 };
 let mut bad:Vec<&str>=q.iter().filter(|(k,v)|!v.is_empty()&&!allowed.contains(&k.as_str())).map(|(k,_)|k.as_str()).collect();
 bad.sort();
 if !bad.is_empty(){return Err(format!("Unsupported parameters (rejected instead of silently dropped): {}",bad.join(", ")));}
 if let Some(m)=q.get("mode"){ if !matches!(m.as_str(),""|"gun"|"auto"){return Err(format!("Unsupported gRPC mode: {m}"));} }
 let sec=q.get("security").map(String::as_str).unwrap_or("");
 let ok=match scheme{"vless"=>matches!(sec,""|"none"|"tls"|"reality"),"trojan"=>matches!(sec,""|"none"|"tls"),_=>true};
 if !ok{return Err(format!("Unsupported security type for {scheme}: {sec}"));}
 Ok(())
}
fn make_outbound(core:&str,uri:&str)->Result<Value,String>{
 let mut v=make_outbound_raw(core,uri)?; let p=parse_uri(uri)?; let q=query(&p.query);
 finish_outbound(&mut v,&p.scheme,&q)?; Ok(v)
}
pub fn convert(uri:&str,core:&str)->Result<Value,String>{let outbound=make_outbound(core,uri)?;match core.to_lowercase().as_str(){"sing-box"|"singbox"=>Ok(json!({"log":{"level":"warn"},"outbounds":[outbound,json!({"type":"direct","tag":"direct"})],"route":{"final":"proxy"}})),"xray"=>{let proto=outbound["type"].as_str().ok_or("Outbound type missing")?;let server=outbound["server"].clone();let port=outbound["server_port"].clone();let settings=match proto{"vless"=>json!({"vnext":[{"address":server,"port":port,"users":[{"id":outbound["uuid"],"encryption":"none","flow":outbound["flow"]}]}]}),"vmess"=>json!({"vnext":[{"address":server,"port":port,"users":[{"id":outbound["uuid"],"alterId":outbound["alter_id"],"security":outbound["security"]}]}]}),"trojan"=>json!({"servers":[{"address":server,"port":port,"password":outbound["password"]}]}),"shadowsocks"=>json!({"servers":[{"address":server,"port":port,"method":outbound["method"],"password":outbound["password"]}]}),_=>return Err(format!("Xray conversion is not implemented for {proto} yet"))};let ptype=match proto{"vless"=>"vless","vmess"=>"vmess","trojan"=>"trojan","shadowsocks"=>"shadowsocks",_=>proto};let mut stream=json!({"network":"tcp","security":"none"});if outbound.get("tls").is_some(){
 if outbound["tls"].get("reality").map(|r|r["enabled"].as_bool()==Some(true)).unwrap_or(false){
  stream["security"]=json!("reality");
  stream["realitySettings"]=json!({"serverName":outbound["tls"]["server_name"],"fingerprint":outbound["tls"]["utls"]["fingerprint"].clone(),"publicKey":outbound["tls"]["reality"]["public_key"],"shortId":outbound["tls"]["reality"]["short_id"],"spiderX":""});
 }else{
  stream["security"]=json!("tls");
  stream["tlsSettings"]=json!({"serverName":outbound["tls"]["server_name"],"allowInsecure":outbound["tls"]["insecure"],"fingerprint":outbound["tls"]["utls"]["fingerprint"].clone(),"alpn":outbound["tls"]["alpn"].clone()});
 }
}if outbound.get("transport").is_some(){let t=outbound["transport"]["type"].as_str().unwrap_or("tcp");stream["network"]=json!(t);if t=="ws"{stream["wsSettings"]=json!({"path":outbound["transport"]["path"],"headers":{"Host":outbound["transport"]["headers"]["Host"]}});}else if t=="grpc"{stream["grpcSettings"]=json!({"serviceName":outbound["transport"]["service_name"]});}else if t=="httpupgrade"{stream["httpupgradeSettings"]=json!({"path":outbound["transport"]["path"],"host":outbound["transport"]["host"]});}else if t!="tcp"{return Err(format!("Xray conversion for transport {t} is not implemented yet"));}}Ok(json!({"log":{"loglevel":"warning"},"inbounds":[],"outbounds":[{"tag":"proxy","protocol":ptype,"settings":settings,"streamSettings":stream},{"tag":"direct","protocol":"freedom","settings":{}},{"tag":"block","protocol":"blackhole","settings":{}}]}))},_=>Err("Unknown core. Use sing-box or xray.".into())}}

#[cfg(test)] mod tests {use super::*;#[test]fn converts_vless_singbox(){let c=convert("vless://1234-uuid@node.example:443?security=tls&type=ws&path=%2Fconnect&sni=example.com#test","sing-box").unwrap();assert_eq!(c["outbounds"][0]["type"],"vless");assert_eq!(c["outbounds"][0]["transport"]["path"],"/connect");}#[test]fn converts_trojan_xray(){let c=convert("trojan://pass@host.test:443?sni=host.test","xray").unwrap();assert_eq!(c["outbounds"][0]["protocol"],"trojan");}#[test]fn converts_raw_shadowsocks_uri(){let c=convert("ss://aes-128-gcm:secret@ss.example.com:8388#raw","sing-box").unwrap();assert_eq!(c["outbounds"][0]["method"],"aes-128-gcm");assert_eq!(c["outbounds"][0]["server_port"],8388);}
#[test]fn xray_vless_reality_uses_reality_settings(){let c=convert("vless://11111111-1111-1111-1111-111111111111@reality.example:443?security=reality&pbk=PUB&sid=abcd&sni=site.example&type=tcp","xray").unwrap();assert_eq!(c["outbounds"][0]["streamSettings"]["security"],"reality");assert_eq!(c["outbounds"][0]["streamSettings"]["realitySettings"]["publicKey"],"PUB");}
#[test]fn rejects_unsupported_scheme(){assert!(convert("ftp://host:21","sing-box").is_err());}#[test]fn converts_ss_uri(){let c=convert("ss://YWVzLTEyOC1nY206c2VjcmV0QHNzLmV4YW1wbGUuY29tOjgzODg#test","sing-box").unwrap();assert_eq!(c["outbounds"][0]["type"],"shadowsocks");assert_eq!(c["outbounds"][0]["server_port"],8388);}}

#[cfg(test)] mod tests_pass8 {use super::*;
#[test]fn vmess_base64_may_contain_slash(){let c=convert("vmess://eyJ2IjoiMiIsInBzIjoidD8/IiwiYWRkIjoidm0uZXhhbXBsZS5jb20iLCJwb3J0IjoiNDQzIiwiaWQiOiIxMTExMTExMS0xMTExLTExMTEtMTExMS0xMTExMTExMTExMTEiLCJhaWQiOiIwIiwibmV0Ijoid3MiLCJ0eXBlIjoibm9uZSIsImhvc3QiOiJ2bS5leGFtcGxlLmNvbSIsInBhdGgiOiIvd3MiLCJ0bHMiOiJ0bHMifQ==","sing-box").unwrap();assert_eq!(c["outbounds"][0]["type"],"vmess");assert_eq!(c["outbounds"][0]["server"],"vm.example.com");assert_eq!(c["outbounds"][0]["transport"]["path"],"/ws");}
#[test]fn plus_in_password_is_preserved(){let c=convert("trojan://pa+ss%2Bword@host.test:443?sni=host.test","sing-box").unwrap();assert_eq!(c["outbounds"][0]["password"],"pa+ss+word");}
#[test]fn percent_decoder_does_not_panic_on_utf8(){let _=decode("%aب");let _=decode("%");let _=decode("%\u{fa}\u{fa}");}
#[test]fn trojan_websocket_transport_is_kept(){let c=convert("trojan://p@host.test:443?type=ws&path=%2Fx&host=h.test","sing-box").unwrap();assert_eq!(c["outbounds"][0]["transport"]["type"],"ws");}
#[test]fn unsupported_parameters_are_rejected_not_dropped(){assert!(convert("vless://u@h.test:443?type=xhttp","sing-box").is_err());assert!(convert("vless://u@h.test:443?encryption=mlkem","sing-box").is_err());}
#[test]fn reality_enables_utls_fingerprint(){let c=convert("vless://u@h.test:443?security=reality&pbk=P&sid=a&sni=s.test&fp=firefox","sing-box").unwrap();assert_eq!(c["outbounds"][0]["tls"]["utls"]["fingerprint"],"firefox");}
}

#[cfg(test)] mod tests_pass9 {use super::*;
#[test]fn unknown_parameters_are_rejected_with_their_names(){let e=convert("hysteria2://pw@h.test:443?sni=a.test&obfs=salamander&obfs-password=x","sing-box").unwrap_err();assert!(e.contains("obfs"));assert!(!e.contains("salamander"));}
#[test]fn ss_plugin_is_rejected_not_dropped(){assert!(convert("ss://aes-128-gcm:s@h.test:8388?plugin=obfs-local","sing-box").is_err());}
#[test]fn unknown_security_is_rejected(){assert!(convert("vless://u@h.test:443?security=xtls","sing-box").is_err());}
#[test]fn common_reality_link_with_spx_still_converts(){let c=convert("vless://u@h.test:443?security=reality&pbk=P&sid=a&sni=s.test&fp=chrome&type=tcp&flow=xtls-rprx-vision&spx=%2F&headerType=none","sing-box").unwrap();assert_eq!(c["outbounds"][0]["tls"]["reality"]["public_key"],"P");}
#[test]fn grpc_gun_mode_is_accepted_other_modes_are_not(){assert!(convert("vless://u@h.test:443?type=grpc&serviceName=s&mode=gun","sing-box").is_ok());assert!(convert("vless://u@h.test:443?type=grpc&serviceName=s&mode=multi","sing-box").is_err());}
#[test]fn trojan_peer_alias_sets_sni(){let c=convert("trojan://p@h.test:443?peer=alias.test","sing-box").unwrap();assert_eq!(c["outbounds"][0]["tls"]["server_name"],"alias.test");}
#[test]fn password_with_special_characters_round_trips(){let c=convert("trojan://p%40ss%3A%2F%23w%2Bd@h.test:443","sing-box").unwrap();assert_eq!(c["outbounds"][0]["password"],"p@ss:/#w+d");}
#[test]fn ipv6_server_is_parsed(){let c=convert("trojan://p@[2001:db8::1]:443","sing-box").unwrap();assert_eq!(c["outbounds"][0]["server"],"2001:db8::1");}
#[test]fn vless_packet_encoding_is_forwarded(){let c=convert("vless://u@h.test:443?security=tls&sni=s.test&packetEncoding=xudp","sing-box").unwrap();assert_eq!(c["outbounds"][0]["packet_encoding"],"xudp");}
#[test]fn tuic_and_wireguard_report_unsupported_instead_of_guessing(){assert!(convert("tuic://u:p@h.test:443","sing-box").unwrap_err().contains("Unsupported protocol"));assert!(convert("wireguard://k@h.test:51820","sing-box").unwrap_err().contains("Unsupported protocol"));}
}
