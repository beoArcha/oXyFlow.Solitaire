//! The system calls: `SetPriorityClass` and `SetThreadPriority` on Windows, `setpriority` on Linux
//! and macOS. Declared by hand like the console calls, so there is no new dependency. On Linux
//! the nice value belongs to a thread: setting it on the UI thread leaves the threads that were
//! started before (the sound output) as they were.

use super::{Flavour, Native, PriorityError};

/// The real system.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemPriority;

#[cfg(windows)]
mod call {
    use std::ffi::c_void;
    use std::io;

    use super::PriorityError;

    const ERROR_ACCESS_DENIED: i32 = 5;
    const THREAD_PRIORITY_BELOW_NORMAL: i32 = -1;

    #[allow(
        unsafe_code,
        reason = "the priority calls have no safe wrapper in std and the crate avoids windows-sys"
    )]
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut c_void;
        fn GetCurrentThread() -> *mut c_void;
        fn SetPriorityClass(process: *mut c_void, class: u32) -> i32;
        fn SetThreadPriority(thread: *mut c_void, priority: i32) -> i32;
    }

    fn failure() -> PriorityError {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(ERROR_ACCESS_DENIED) {
            PriorityError::Denied(error)
        } else {
            PriorityError::Refused(error)
        }
    }

    #[allow(
        unsafe_code,
        reason = "SetPriorityClass has no safe wrapper in std and the crate avoids windows-sys"
    )]
    pub(super) fn set_process(value: i32) -> Result<(), PriorityError> {
        let class = u32::try_from(value).map_err(|_error| PriorityError::Unsupported)?;
        // SAFETY: the pseudo handle needs no closing; the class is a documented non-real-time one.
        let done = unsafe { SetPriorityClass(GetCurrentProcess(), class) };
        if done == 0 { Err(failure()) } else { Ok(()) }
    }

    #[allow(
        unsafe_code,
        reason = "SetThreadPriority has no safe wrapper in std and the crate avoids windows-sys"
    )]
    pub(super) fn lower_current_thread() -> Result<(), PriorityError> {
        // SAFETY: the pseudo handle needs no closing; the priority is THREAD_PRIORITY_BELOW_NORMAL.
        let done = unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL) };
        if done == 0 { Err(failure()) } else { Ok(()) }
    }
}

#[cfg(unix)]
mod call {
    use std::io;

    use super::PriorityError;
    #[cfg(target_os = "linux")]
    use crate::priority::SOLVER_NICE;

    const PRIO_PROCESS: i32 = 0;
    const EPERM: i32 = 1;
    const EACCES: i32 = 13;

    #[allow(
        unsafe_code,
        reason = "setpriority has no safe wrapper in std and the crate avoids libc"
    )]
    unsafe extern "C" {
        fn setpriority(which: i32, who: u32, priority: i32) -> i32;
    }

    fn failure() -> PriorityError {
        let error = io::Error::last_os_error();
        match error.raw_os_error() {
            Some(EPERM | EACCES) => PriorityError::Denied(error),
            _ => PriorityError::Refused(error),
        }
    }

    #[allow(
        unsafe_code,
        reason = "setpriority has no safe wrapper in std and the crate avoids libc"
    )]
    fn set_nice(nice: i32) -> Result<(), PriorityError> {
        // SAFETY: setpriority only reads its three integer arguments; who 0 names the caller.
        let done = unsafe { setpriority(PRIO_PROCESS, 0, nice) };
        if done == 0 { Ok(()) } else { Err(failure()) }
    }

    pub(super) fn set_process(value: i32) -> Result<(), PriorityError> {
        set_nice(value)
    }

    #[cfg(target_os = "linux")]
    pub(super) fn lower_current_thread() -> Result<(), PriorityError> {
        set_nice(SOLVER_NICE)
    }

    #[cfg(not(target_os = "linux"))]
    pub(super) fn lower_current_thread() -> Result<(), PriorityError> {
        Err(PriorityError::Unsupported)
    }
}

#[cfg(not(any(windows, unix)))]
mod call {
    use super::PriorityError;

    pub(super) fn set_process(_value: i32) -> Result<(), PriorityError> {
        Err(PriorityError::Unsupported)
    }

    pub(super) fn lower_current_thread() -> Result<(), PriorityError> {
        Err(PriorityError::Unsupported)
    }
}

impl Native for SystemPriority {
    fn flavour(&self) -> Flavour {
        if cfg!(windows) {
            Flavour::Windows
        } else {
            Flavour::Unix
        }
    }

    fn set_process(&self, value: i32) -> Result<(), PriorityError> {
        call::set_process(value)
    }

    fn lower_current_thread(&self) -> Result<(), PriorityError> {
        call::lower_current_thread()
    }
}
