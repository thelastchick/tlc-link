//! Shared Kill Switch primitives (journal + WFP) used by the UI process and the
//! optional independent helper service `tlc-ks-service`.

pub mod journal;
pub mod protocol;

#[cfg(windows)]
pub mod wfp;

#[cfg(not(windows))]
pub mod wfp {
    use std::net::IpAddr;
    pub fn install_protection(_: &IpAddr, _: u16, _: Option<&IpAddr>) -> Result<Vec<u64>, String> {
        Err("WFP فقط روی Windows در دسترس است".into())
    }
    pub fn install_hard_block() -> Result<Vec<u64>, String> {
        Err("WFP فقط روی Windows در دسترس است".into())
    }
    pub fn remove_filters_by_ids(_: &[u64]) -> Result<(), String> {
        Ok(())
    }
    pub fn remove_all_tlc_objects() -> Result<u32, String> {
        Ok(0)
    }
}
