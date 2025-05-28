use core::ffi::{c_int, c_void};

use axerrno::{LinuxError, LinuxResult};
use axtask::{TaskExtRef, current, futex_sleep, futex_wake};

use crate::ptr::{PtrWrapper, UserConstPtr, UserPtr};

use arceos_posix_api as api;
use arceos_posix_api::ctypes::rlimit;
use arceos_posix_api::ctypes::timespec;

use starry_core::mm::AddrSpace;
use starry_core::signal::{self, SigMask, Signal};

pub fn sys_rt_sigprocmask(
    _how: i32,
    _set: UserConstPtr<c_void>,
    _oldset: UserPtr<c_void>,
    _sigsetsize: usize,
) -> LinuxResult<isize> {
    warn!("sys_rt_sigprocmask: not implemented");
    Ok(0)
}

// TODO
pub fn sys_rt_sigaction(
    _signum: i32,
    _act: UserConstPtr<c_void>,
    _oldact: UserPtr<c_void>,
    _sigsetsize: usize,
) -> LinuxResult<isize> {
    warn!("sys_rt_sigaction: not implemented");
    Ok(0)
}

const FUTEX_WAIT: c_int = 0;
const FUTEX_WAKE: c_int = 1;

pub fn sys_futex(
    uaddr: UserPtr<i32>,
    futex_op: c_int,
    val: c_int,
    timeout: UserPtr<timespec>,
    uaddr2: UserPtr<i32>,
    val3: c_int,
) -> LinuxResult<isize> {
    let futex_key_ptr: *mut i32 = uaddr.get().unwrap();
    let futex_key = unsafe { *futex_key_ptr };
    if futex_op == FUTEX_WAIT {
        futex_sleep(futex_key);
        Ok(0)
    } else if futex_op == FUTEX_WAKE {
        Ok(futex_wake(futex_key) as isize)
    } else {
        Err(LinuxError::EINVAL)
    }
}

pub fn sys_rt_kill(pid: c_int, sig: c_int) -> LinuxResult<isize> {
    signal::send_signal_proc(pid, sig)
}

pub fn sys_tkill(tid: c_int, sig: c_int) -> LinuxResult<isize> {
    signal::send_signal_thread(tid, sig)
}

pub fn sys_tgkill(tgid: c_int, tid: c_int, sig: c_int) -> LinuxResult<isize> {
    signal::send_signal_thread(tid, sig)
}

pub fn sys_rt_sigtimedwait() -> LinuxResult<isize> {
    warn!("sys_rt_sigtimedwait: I'm always waiting for you.");
    Ok(0)
}

pub fn sys_rt_getrlimit(resource: c_int, rlimits: UserPtr<rlimit>) -> LinuxResult<isize> {
    Ok(unsafe {
        api::sys_getrlimit(resource, rlimits.get()?)
            .try_into()
            .unwrap()
    })
}

pub fn sys_rt_setrlimit(resource: c_int, rlimits: UserPtr<rlimit>) -> LinuxResult<isize> {
    Ok(unsafe {
        api::sys_setrlimit(resource, rlimits.get()?)
            .try_into()
            .unwrap()
    })
}
