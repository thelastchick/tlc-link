//! Windows process helpers: Job Object (children die with the app), process identity
//! checks and identity-checked termination. On other platforms these are inert stubs so
//! the crate (and its logic tests) still compile.
//!
//! Not compiled or run in the authoring environment (no Rust toolchain, no Windows).

/// Normalises a Windows path for comparison (case-insensitive, `/` == `\`).
pub fn same_path(a: &str, b: &str) -> bool {
    let n = |s: &str| s.replace('/', "\\").trim_end_matches('\\').to_ascii_lowercase();
    n(a) == n(b)
}

#[cfg(windows)]
mod imp {
    use std::os::windows::io::AsRawHandle;
    use std::sync::OnceLock;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, TerminateProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
    };

    struct Job(HANDLE);
    // The handle is only ever used for Assign..., and closed by the OS when the app dies.
    unsafe impl Send for Job {}
    unsafe impl Sync for Job {}

    static JOB: OnceLock<Option<Job>> = OnceLock::new();

    fn create_job() -> Option<Job> {
        unsafe {
            let h = CreateJobObjectW(core::ptr::null(), core::ptr::null());
            if h.is_null() { return None; }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = core::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let ok = SetInformationJobObject(
                h,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const core::ffi::c_void,
                core::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            if ok == 0 { CloseHandle(h); return None; }
            Some(Job(h))
        }
    }

    /// Puts the child into a kill-on-close Job Object. When TLC LINK exits for ANY reason
    /// (crash, Task Manager kill) the OS closes the job handle and terminates the core.
    pub fn attach_to_kill_on_close_job(child: &std::process::Child) -> Result<(), String> {
        let job = JOB.get_or_init(create_job).as_ref().ok_or("ساخت Job Object ناموفق بود")?;
        let ok = unsafe { AssignProcessToJobObject(job.0, child.as_raw_handle() as HANDLE) };
        if ok == 0 { Err("افزودن هسته به Job Object ناموفق بود".into()) } else { Ok(()) }
    }

    pub fn image_path(pid: u32) -> Option<String> {
        unsafe {
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h.is_null() { return None; }
            let mut buf = [0u16; 2048];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut len);
            CloseHandle(h);
            if ok == 0 { None } else { Some(String::from_utf16_lossy(&buf[..len as usize])) }
        }
    }

    /// Terminates `pid` only if its full executable path equals `expected_path`.
    /// Ok(true)=terminated, Ok(false)=process gone or belongs to someone else (left alone).
    pub fn kill_if_image_matches(pid: u32, expected_path: &str) -> Result<bool, String> {
        match image_path(pid) {
            None => Ok(false),
            Some(actual) if !super::same_path(&actual, expected_path) => Ok(false),
            Some(_) => unsafe {
                let h = OpenProcess(PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
                if h.is_null() { return Err(format!("باز کردن پردازش {pid} برای توقف ممکن نشد")); }
                let ok = TerminateProcess(h, 1);
                CloseHandle(h);
                if ok == 0 { Err(format!("توقف پردازش {pid} ناموفق بود")) } else { Ok(true) }
            },
        }
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn attach_to_kill_on_close_job(_: &std::process::Child) -> Result<(), String> { Err("Job Object فقط روی ویندوز وجود دارد".into()) }
    pub fn image_path(_: u32) -> Option<String> { None }
    pub fn kill_if_image_matches(_: u32, _: &str) -> Result<bool, String> { Ok(false) }
}

pub use imp::{attach_to_kill_on_close_job, image_path, kill_if_image_matches};

#[cfg(test)]
mod tests {
    use super::same_path;
    #[test]
    fn path_comparison_ignores_case_and_separators() {
        assert!(same_path(r"C:\App\Sing-Box.exe", "c:/app/sing-box.EXE"));
        assert!(!same_path(r"C:\App\sing-box.exe", r"C:\Other\sing-box.exe"));
    }
}
