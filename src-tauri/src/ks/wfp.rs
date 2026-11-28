//! Windows Filtering Platform backend for TLC LINK Kill Switch.
#![allow(non_snake_case, dead_code)]

use crate::ks::journal::{TLC_PROVIDER_KEY, TLC_SUBLAYER_KEY};
use std::{net::IpAddr, ptr};
use windows_sys::core::GUID;
use windows_sys::Win32::Foundation::{ERROR_SUCCESS, HANDLE};
use windows_sys::Win32::NetworkManagement::WindowsFilteringPlatform::*;
use windows_sys::Win32::System::Rpc::RPC_C_AUTHN_WINNT;

fn parse_guid(s: &str) -> Result<GUID, String> {
    // Accept {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}
    let t = s.trim().trim_start_matches('{').trim_end_matches('}');
    let parts: Vec<&str> = t.split('-').collect();
    if parts.len() != 5 {
        return Err(format!("GUID نامعتبر: {s}"));
    }
    let data1 = u32::from_str_radix(parts[0], 16).map_err(|_| "GUID data1")?;
    let data2 = u16::from_str_radix(parts[1], 16).map_err(|_| "GUID data2")?;
    let data3 = u16::from_str_radix(parts[2], 16).map_err(|_| "GUID data3")?;
    let rest = format!("{}{}", parts[3], parts[4]);
    if rest.len() != 16 {
        return Err("GUID data4 length".into());
    }
    let mut data4 = [0u8; 8];
    for i in 0..8 {
        data4[i] = u8::from_str_radix(&rest[i * 2..i * 2 + 2], 16)
            .map_err(|_| "GUID data4 byte")?;
    }
    Ok(GUID {
        data1: data1,
        data2: data2,
        data3: data3,
        data4: data4,
    })
}

struct Engine {
    handle: HANDLE,
}

impl Engine {
    fn open() -> Result<Self, String> {
        let mut handle: HANDLE = ptr::null_mut();
        let mut session: FWPM_SESSION0 = unsafe { std::mem::zeroed() };
        // Non-dynamic so filters persist after process exit until we remove them.
        session.flags = 0;
        let status = unsafe {
            FwpmEngineOpen0(
                ptr::null(),
                RPC_C_AUTHN_WINNT,
                ptr::null(),
                &session,
                &mut handle,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(format!(
                "FwpmEngineOpen0 failed: 0x{status:08X} (نیاز به اجرای مدیر / سرویس BFE)"
            ));
        }
        Ok(Self { handle })
    }

    fn begin_transaction(&self) -> Result<(), String> {
        let status = unsafe { FwpmTransactionBegin0(self.handle, 0) };
        if status != ERROR_SUCCESS {
            return Err(format!("FwpmTransactionBegin0: 0x{status:08X}"));
        }
        Ok(())
    }

    fn commit(&self) -> Result<(), String> {
        let status = unsafe { FwpmTransactionCommit0(self.handle) };
        if status != ERROR_SUCCESS {
            return Err(format!("FwpmTransactionCommit0: 0x{status:08X}"));
        }
        Ok(())
    }

    fn abort(&self) {
        let _ = unsafe { FwpmTransactionAbort0(self.handle) };
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { FwpmEngineClose0(self.handle) };
        }
    }
}

fn ensure_provider_and_sublayer(engine: &Engine) -> Result<(), String> {
    let provider_key = parse_guid(TLC_PROVIDER_KEY)?;
    let sublayer_key = parse_guid(TLC_SUBLAYER_KEY)?;

    // Provider
    let mut provider: FWPM_PROVIDER0 = unsafe { std::mem::zeroed() };
    provider.providerKey = provider_key;
    let name: Vec<u16> = "TLC LINK Kill Switch\0".encode_utf16().collect();
    let desc: Vec<u16> = "TLC LINK connection-scoped WFP provider\0"
        .encode_utf16()
        .collect();
    provider.displayData.name = name.as_ptr() as *mut u16;
    provider.displayData.description = desc.as_ptr() as *mut u16;
    // Ignore ALREADY_EXISTS
    let st = unsafe { FwpmProviderAdd0(engine.handle, &provider, ptr::null_mut()) };
    if st != ERROR_SUCCESS && st != 0x80320009 {
        // FWP_E_ALREADY_EXISTS
        return Err(format!("FwpmProviderAdd0: 0x{st:08X}"));
    }

    // Sublayer (high weight so our blocks are evaluated early)
    let mut sub: FWPM_SUBLAYER0 = unsafe { std::mem::zeroed() };
    sub.subLayerKey = sublayer_key;
    let mut pk = provider_key;
    sub.providerKey = &mut pk;
    let sname: Vec<u16> = "TLC LINK KS Sublayer\0".encode_utf16().collect();
    sub.displayData.name = sname.as_ptr() as *mut u16;
    sub.weight = 0xFFFF; // max
    let st = unsafe { FwpmSubLayerAdd0(engine.handle, &sub, ptr::null_mut()) };
    if st != ERROR_SUCCESS && st != 0x80320009 {
        return Err(format!("FwpmSubLayerAdd0: 0x{st:08X}"));
    }
    Ok(())
}

fn add_block_all_filter(
    engine: &Engine,
    layer: GUID,
    name: &str,
    weight: u64,
) -> Result<u64, String> {
    let sublayer_key = parse_guid(TLC_SUBLAYER_KEY)?;
    let provider_key = parse_guid(TLC_PROVIDER_KEY)?;
    let mut filter: FWPM_FILTER0 = unsafe { std::mem::zeroed() };
    // name_buf must outlive the FwpmFilterAdd0 call
    let name_buf: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    filter.displayData.name = name_buf.as_ptr() as *mut u16;
    filter.layerKey = layer;
    filter.subLayerKey = sublayer_key;
    let mut pk = provider_key;
    filter.providerKey = &mut pk;
    filter.action.r#type = FWP_ACTION_BLOCK;
    filter.weight.r#type = FWP_UINT64;
    // weight value stored as *mut u64 per FWP_VALUE0 layout
    let mut w = weight;
    unsafe {
        filter.weight.Anonymous.uint64 = &mut w;
    }
    filter.numFilterConditions = 0;
    filter.filterCondition = ptr::null_mut();
    // Persistent so filters survive UI process exit until explicit release
    filter.flags = FWPM_FILTER_FLAG_PERSISTENT;

    let mut id: u64 = 0;
    let st = unsafe { FwpmFilterAdd0(engine.handle, &filter, ptr::null_mut(), &mut id) };
    // name_buf must stay alive for the duration of the API call
    drop(name_buf);
    if st != ERROR_SUCCESS {
        return Err(format!("FwpmFilterAdd0 (block): 0x{st:08X}"));
    }
    Ok(id)
}

fn add_permit_remote(
    engine: &Engine,
    layer: GUID,
    ip: &IpAddr,
    port: u16,
    name: &str,
    weight: u64,
) -> Result<u64, String> {
    let sublayer_key = parse_guid(TLC_SUBLAYER_KEY)?;
    let provider_key = parse_guid(TLC_PROVIDER_KEY)?;

    // Conditions: remote IP + remote port
    let mut conditions: Vec<FWPM_FILTER_CONDITION0> = Vec::new();

    // Remote address
    let mut cond_addr: FWPM_FILTER_CONDITION0 = unsafe { std::mem::zeroed() };
    cond_addr.fieldKey = FWPM_CONDITION_IP_REMOTE_ADDRESS;
    cond_addr.matchType = FWP_MATCH_EQUAL;
    // Keep V6 array alive for the duration of FwpmFilterAdd0
    let mut v6_storage: Option<Box<FWP_BYTE_ARRAY16>> = None;
    match ip {
        IpAddr::V4(v4) => {
            // Network byte order (big-endian) as required by WFP for V4 addresses
            let addr = u32::from_be_bytes(v4.octets());
            cond_addr.conditionValue.r#type = FWP_UINT32;
            unsafe {
                cond_addr.conditionValue.Anonymous.uint32 = addr;
            }
        }
        IpAddr::V6(v6) => {
            let mut arr: FWP_BYTE_ARRAY16 = unsafe { std::mem::zeroed() };
            arr.byteArray16 = v6.octets();
            let boxed = Box::new(arr);
            cond_addr.conditionValue.r#type = FWP_BYTE_ARRAY16_TYPE;
            unsafe {
                cond_addr.conditionValue.Anonymous.byteArray16 = boxed.as_ref() as *const _ as *mut _;
            }
            v6_storage = Some(boxed);
        }
    }
    conditions.push(cond_addr);

    // Remote port
    let mut cond_port: FWPM_FILTER_CONDITION0 = unsafe { std::mem::zeroed() };
    cond_port.fieldKey = FWPM_CONDITION_IP_REMOTE_PORT;
    cond_port.matchType = FWP_MATCH_EQUAL;
    cond_port.conditionValue.r#type = FWP_UINT16;
    unsafe {
        cond_port.conditionValue.Anonymous.uint16 = port;
    }
    conditions.push(cond_port);

    let name_buf: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let mut filter: FWPM_FILTER0 = unsafe { std::mem::zeroed() };
    filter.displayData.name = name_buf.as_ptr() as *mut u16;
    filter.layerKey = layer;
    filter.subLayerKey = sublayer_key;
    let mut pk = provider_key;
    filter.providerKey = &mut pk;
    filter.action.r#type = FWP_ACTION_PERMIT;
    filter.weight.r#type = FWP_UINT64;
    let mut w = weight;
    unsafe {
        filter.weight.Anonymous.uint64 = &mut w;
    }
    filter.numFilterConditions = conditions.len() as u32;
    filter.filterCondition = conditions.as_mut_ptr();
    filter.flags = FWPM_FILTER_FLAG_PERSISTENT;

    let mut id: u64 = 0;
    let st = unsafe { FwpmFilterAdd0(engine.handle, &filter, ptr::null_mut(), &mut id) };
    drop(name_buf);
    drop(v6_storage);
    if st != ERROR_SUCCESS {
        return Err(format!("FwpmFilterAdd0 (permit): 0x{st:08X}"));
    }
    Ok(id)
}

/// Install block-all + permit endpoint (and optional previous endpoint during switch).
pub fn install_protection(
    endpoint_ip: &IpAddr,
    port: u16,
    previous_ip: Option<&IpAddr>,
) -> Result<Vec<u64>, String> {
    if cfg!(test) { return Ok(vec![10, 11, 12]); }

    let engine = Engine::open()?;
    engine.begin_transaction()?;
    if let Err(e) = ensure_provider_and_sublayer(&engine) {
        engine.abort();
        return Err(e);
    }

    let mut ids = Vec::new();
    // Layers: ALE AUTH_CONNECT V4 / V6 cover TCP/UDP outbound connect decisions.
    let layers_v4 = [FWPM_LAYER_ALE_AUTH_CONNECT_V4];
    let layers_v6 = [FWPM_LAYER_ALE_AUTH_CONNECT_V6];

    for layer in layers_v4 {
        // High-weight permit for endpoint first
        match add_permit_remote(
            &engine,
            layer,
            endpoint_ip,
            port,
            "TLC KS permit endpoint V4",
            15,
        ) {
            Ok(id) => ids.push(id),
            Err(e) => {
                engine.abort();
                return Err(e);
            }
        }
        if let Some(old) = previous_ip {
            if let Ok(id) = add_permit_remote(
                &engine,
                layer,
                old,
                port,
                "TLC KS permit previous endpoint V4",
                14,
            ) {
                ids.push(id);
            }
        }
        // Lower-weight block-all
        match add_block_all_filter(&engine, layer, "TLC KS block all V4", 1) {
            Ok(id) => ids.push(id),
            Err(e) => {
                engine.abort();
                return Err(e);
            }
        }
    }

    // IPv6 block always; permit only if endpoint is V6
    for layer in layers_v6 {
        if matches!(endpoint_ip, IpAddr::V6(_)) {
            if let Ok(id) = add_permit_remote(
                &engine,
                layer,
                endpoint_ip,
                port,
                "TLC KS permit endpoint V6",
                15,
            ) {
                ids.push(id);
            }
        }
        match add_block_all_filter(&engine, layer, "TLC KS block all V6", 1) {
            Ok(id) => ids.push(id),
            Err(e) => {
                engine.abort();
                return Err(e);
            }
        }
    }

    engine.commit()?;
    Ok(ids)
}

/// Hard block with no endpoint permit (post-crash / boot reconcile).
pub fn install_hard_block() -> Result<Vec<u64>, String> {
    if cfg!(test) { return Ok(vec![1, 2]); }

    let engine = Engine::open()?;
    engine.begin_transaction()?;
    if let Err(e) = ensure_provider_and_sublayer(&engine) {
        engine.abort();
        return Err(e);
    }
    let mut ids = Vec::new();
    for layer in [FWPM_LAYER_ALE_AUTH_CONNECT_V4, FWPM_LAYER_ALE_AUTH_CONNECT_V6] {
        match add_block_all_filter(&engine, layer, "TLC KS hard block", 1) {
            Ok(id) => ids.push(id),
            Err(e) => {
                engine.abort();
                return Err(e);
            }
        }
    }
    engine.commit()?;
    Ok(ids)
}

pub fn remove_filters_by_ids(ids: &[u64]) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    let engine = Engine::open()?;
    engine.begin_transaction()?;
    for id in ids {
        let st = unsafe { FwpmFilterDeleteById0(engine.handle, *id) };
        // Ignore not-found
        if st != ERROR_SUCCESS && st != 0x80320003 {
            // FWP_E_FILTER_NOT_FOUND
            engine.abort();
            return Err(format!("FwpmFilterDeleteById0({id}): 0x{st:08X}"));
        }
    }
    engine.commit()?;
    Ok(())
}

/// Enumerate and delete every filter/sublayer/provider owned by TLC keys.
pub fn remove_all_tlc_objects() -> Result<u32, String> {
    // Unit tests must not touch the real WFP engine (heap/privilege side effects on CI).
    if cfg!(test) { return Ok(0); }

    let engine = Engine::open()?;
    engine.begin_transaction()?;
    let provider_key = parse_guid(TLC_PROVIDER_KEY)?;
    let sublayer_key = parse_guid(TLC_SUBLAYER_KEY)?;
    let mut removed: u32 = 0;

    // Delete filters by provider (enum + delete)
    // Simplified path: delete sublayer then provider; Windows removes associated filters
    // when provider is deleted in some versions — we still try filter enum.
    let mut enum_handle: HANDLE = ptr::null_mut();
    let st = unsafe {
        FwpmFilterCreateEnumHandle0(engine.handle, ptr::null(), &mut enum_handle)
    };
    if st == ERROR_SUCCESS {
        loop {
            let mut entries: *mut *mut FWPM_FILTER0 = ptr::null_mut();
            let mut count: u32 = 0;
            let st = unsafe {
                FwpmFilterEnum0(engine.handle, enum_handle, 64, &mut entries, &mut count)
            };
            if st != ERROR_SUCCESS || count == 0 {
                break;
            }
            for i in 0..count as isize {
                let f = unsafe { &**entries.offset(i) };
                let is_ours = !f.providerKey.is_null()
                    && unsafe { *f.providerKey }.data1 == provider_key.data1
                    && unsafe { *f.providerKey }.data2 == provider_key.data2;
                if is_ours {
                    let del = unsafe { FwpmFilterDeleteById0(engine.handle, f.filterId) };
                    if del == ERROR_SUCCESS {
                        removed += 1;
                    }
                }
            }
            unsafe { FwpmFreeMemory0(entries as *mut _) };
        }
        unsafe { FwpmFilterDestroyEnumHandle0(engine.handle, enum_handle) };
    }

    let st = unsafe { FwpmSubLayerDeleteByKey0(engine.handle, &sublayer_key) };
    if st == ERROR_SUCCESS {
        removed += 1;
    }
    let st = unsafe { FwpmProviderDeleteByKey0(engine.handle, &provider_key) };
    if st == ERROR_SUCCESS {
        removed += 1;
    }

    engine.commit()?;
    Ok(removed)
}

