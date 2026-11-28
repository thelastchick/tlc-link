//! TLC LINK Kill Switch independent helper.
//!
//! Owns WFP filter lifetime so protection can outlive the UI process.
//! Protocol: length-prefixed JSON over named pipe `\\.\pipe\tlc-ks-svc-v1`
//! (see `tlc_link_lib::ks::protocol`).
//!
//! Security note (current limitations — still open):
//! - Named pipe is created with default DACL (any local process can connect).
//!   A future pass should restrict the ACL to Administrators / the installing user
//!   and optionally require a shared secret written at install time.
//! - Status is process-local; it does not re-enumerate WFP filters on every Status call.
//!
//! Usage (Administrator):
//!   tlc-ks-service.exe
//! Install as Windows service (optional):
//!   sc create TLCKillSwitch binPath= "C:\full\path\tlc-ks-service.exe" start= auto
//!   sc start TLCKillSwitch

fn main() {
    #[cfg(not(windows))]
    {
        eprintln!("tlc-ks-service is Windows-only");
        std::process::exit(1);
    }
    #[cfg(windows)]
    if let Err(e) = windows_main() {
        eprintln!("tlc-ks-service error: {e}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn windows_main() -> Result<(), String> {
    use std::io::{Read, Write};
    use std::net::IpAddr;
    use std::ptr;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use tlc_link_lib::ks::protocol::{KsRequest, KsResponse, PIPE_NAME, PROTOCOL_VERSION};
    use tlc_link_lib::ks::wfp;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Pipes::{
        CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE,
        PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
    };
    // windows-sys feature layout varies across minor versions; bind the stable kernel32
    // entry points we need so the helper binary always builds on CI.
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn ConnectNamedPipe(h: HANDLE, overlapped: *mut core::ffi::c_void) -> i32;
    }

    // PIPE_ACCESS_DUPLEX = 0x00000003
    const PIPE_ACCESS_DUPLEX: u32 = 0x0000_0003;

    // Process-local armed tracking. Good enough for Status replies until we
    // re-enumerate WFP filters on every status request.
    static ARMED: AtomicBool = AtomicBool::new(false);
    static GENERATION: AtomicU64 = AtomicU64::new(0);

    eprintln!("TLC LINK KS service listening on {PIPE_NAME}");

    loop {
        let wide: Vec<u16> = PIPE_NAME.encode_utf16().chain(Some(0)).collect();
        let handle: HANDLE = unsafe {
            CreateNamedPipeW(
                wide.as_ptr(),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                65536,
                65536,
                0,
                ptr::null(),
            )
        };
        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            return Err(format!(
                "CreateNamedPipeW: {}",
                std::io::Error::last_os_error()
            ));
        }
        let ok = unsafe { ConnectNamedPipe(handle, ptr::null_mut()) };
        if ok == 0 {
            let err = std::io::Error::last_os_error();
            // 535 = ERROR_PIPE_CONNECTED
            if err.raw_os_error() != Some(535) {
                unsafe { CloseHandle(handle) };
                continue;
            }
        }

        let mut pipe = PipeHandle { handle };
        let mut len_buf = [0u8; 4];
        if pipe.read_exact(&mut len_buf).is_err() {
            continue;
        }
        let n = u32::from_le_bytes(len_buf) as usize;
        if n == 0 || n > 1_000_000 {
            continue;
        }
        let mut body = vec![0u8; n];
        if pipe.read_exact(&mut body).is_err() {
            continue;
        }

        let resp = match serde_json::from_slice::<KsRequest>(&body) {
            Ok(KsRequest::Ping { version }) => {
                if version == PROTOCOL_VERSION {
                    KsResponse::Ok {
                        message: "pong".into(),
                    }
                } else {
                    KsResponse::Err {
                        message: format!("protocol {version} != {PROTOCOL_VERSION}"),
                    }
                }
            }
            Ok(KsRequest::Arm {
                port,
                ip,
                previous_ip,
                ..
            }) => match ip.parse::<IpAddr>() {
                Ok(addr) => {
                    let prev = previous_ip.and_then(|p| p.parse().ok());
                    match wfp::install_protection(&addr, port, prev.as_ref()) {
                        Ok(ids) => {
                            ARMED.store(true, Ordering::SeqCst);
                            GENERATION.fetch_add(1, Ordering::SeqCst);
                            KsResponse::Armed {
                                filter_ids: ids,
                                message: "armed".into(),
                            }
                        }
                        Err(e) => KsResponse::Err { message: e },
                    }
                }
                Err(_) => KsResponse::Err {
                    message: "invalid ip".into(),
                },
            },
            Ok(KsRequest::Release) | Ok(KsRequest::EmergencyClear) => {
                match wfp::remove_all_tlc_objects() {
                    Ok(n) => {
                        ARMED.store(false, Ordering::SeqCst);
                        KsResponse::Ok {
                            message: format!("cleared {n} objects"),
                        }
                    }
                    Err(e) => KsResponse::Err { message: e },
                }
            }
            Ok(KsRequest::Status) => KsResponse::Status {
                state: if ARMED.load(Ordering::SeqCst) {
                    "armed".into()
                } else {
                    "idle".into()
                },
                armed: ARMED.load(Ordering::SeqCst),
                generation: GENERATION.load(Ordering::SeqCst),
                service: true,
            },
            Err(e) => KsResponse::Err {
                message: format!("bad json: {e}"),
            },
        };

        if let Ok(payload) = serde_json::to_vec(&resp) {
            let len = (payload.len() as u32).to_le_bytes();
            let _ = pipe.write_all(&len);
            let _ = pipe.write_all(&payload);
            let _ = pipe.flush();
        }
        // Drop disconnects the client.
    }
}

#[cfg(windows)]
struct PipeHandle {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn ReadFile(
        h: windows_sys::Win32::Foundation::HANDLE,
        buf: *mut u8,
        n: u32,
        read: *mut u32,
        overlapped: *mut core::ffi::c_void,
    ) -> i32;
    fn WriteFile(
        h: windows_sys::Win32::Foundation::HANDLE,
        buf: *const u8,
        n: u32,
        written: *mut u32,
        overlapped: *mut core::ffi::c_void,
    ) -> i32;
    fn FlushFileBuffers(h: windows_sys::Win32::Foundation::HANDLE) -> i32;
}

#[cfg(windows)]
impl std::io::Read for PipeHandle {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut read = 0u32;
        let ok = unsafe {
            ReadFile(
                self.handle,
                buf.as_mut_ptr(),
                buf.len() as u32,
                &mut read,
                core::ptr::null_mut(),
            )
        };
        if ok == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(read as usize)
        }
    }
}

#[cfg(windows)]
impl std::io::Write for PipeHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut written = 0u32;
        let ok = unsafe {
            WriteFile(
                self.handle,
                buf.as_ptr(),
                buf.len() as u32,
                &mut written,
                core::ptr::null_mut(),
            )
        };
        if ok == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(written as usize)
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        let ok = unsafe { FlushFileBuffers(self.handle) };
        if ok == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

#[cfg(windows)]
impl Drop for PipeHandle {
    fn drop(&mut self) {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Pipes::DisconnectNamedPipe;
        unsafe {
            DisconnectNamedPipe(self.handle);
            CloseHandle(self.handle);
        }
    }
}
