use core::ffi::{c_int, c_void};

use axerrno::{LinuxError, LinuxResult};
use axtask::{TaskExtMut, TaskExtRef, current, futex_sleep, futex_wake};
use starry_core::task::ProcessData;

use crate::ptr::{PtrWrapper, UserConstPtr, UserPtr};

use arceos_posix_api as api;
use arceos_posix_api::ctypes::rlimit;
use arceos_posix_api::ctypes::timespec;

use starry_core::mm::AddrSpace;
use starry_core::signal::{self, SigMask, Signal};

use api::ctypes;
use axconfig;

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

/// Get resource limitations
///
/// TODO: support more resource types
pub unsafe fn sys_getrlimit(resource: c_int, rlimits: *mut ctypes::rlimit) -> c_int {
    debug!("sys_getrlimit <= {} {:#x}", resource, rlimits as usize);
    match resource as u32 {
        ctypes::RLIMIT_DATA => {}
        ctypes::RLIMIT_STACK => {}
        ctypes::RLIMIT_NOFILE => {}
        _ => return LinuxError::EINVAL as _,
    }
    if rlimits.is_null() {
        return 0;
    }
    match resource as u32 {
        ctypes::RLIMIT_STACK => unsafe {
            (*rlimits).rlim_cur = axconfig::TASK_STACK_SIZE as _;
            (*rlimits).rlim_max = axconfig::TASK_STACK_SIZE as _;
        },
        //#[cfg(feature = "fd")]
        ctypes::RLIMIT_NOFILE => unsafe {
            let curr = axtask::current();
            let data = curr.task_ext().process_data();
            (*rlimits) = *data.fd_limit.lock();
            debug!(
                "got rlimits: {} / {}",
                (*rlimits).rlim_cur,
                (*rlimits).rlim_max
            );
        },
        _ => {}
    }
    0
}

/// Set resource limitations
///
/// TODO: support more resource types
pub unsafe fn sys_setrlimit(resource: c_int, rlimits: *mut ctypes::rlimit) -> c_int {
    debug!("sys_setrlimit <= {} {:#x}", resource, rlimits as usize);
    let rlimits = unsafe { rlimits.read() };

    // TODO: check permission
    // assert!(rlimits.rlim_cur < rlimits.rlim_max);
    match resource as u32 {
        ctypes::RLIMIT_DATA => {}
        ctypes::RLIMIT_STACK => {}
        ctypes::RLIMIT_NOFILE => {
            let curr = axtask::current();
            let data: &ProcessData = curr.task_ext().process_data();
            {
                // 限制锁范围
                let mut fd_limit_guard = data.fd_limit.lock();
                *fd_limit_guard = rlimits;
                debug!(
                    "changed fd_limit: {} / {}",
                    fd_limit_guard.rlim_cur, fd_limit_guard.rlim_max
                );
            }
        }
        _ => return LinuxError::EINVAL as _,
    }
    // Currently do not support set resources
    0
}
pub fn sys_rt_getrlimit(resource: c_int, rlimits: UserPtr<rlimit>) -> LinuxResult<isize> {
    Ok(unsafe { sys_getrlimit(resource, rlimits.get()?).try_into().unwrap() })
}

pub fn sys_rt_setrlimit(resource: c_int, rlimits: UserPtr<rlimit>) -> LinuxResult<isize> {
    Ok(unsafe { sys_setrlimit(resource, rlimits.get()?).try_into().unwrap() })
}

pub fn sys_rt_prlimit64(
    pid: c_int,
    resource: c_int,
    new_rlimits: UserPtr<rlimit>,
    old_rlimits: UserPtr<rlimit>,
) -> LinuxResult<isize> {
    if pid != 0 {
        debug!("sys_rt_prlimit64: Operations on PID {} not supported.", pid);
        return Err(LinuxError::EPERM);
    }

    let get_result = sys_rt_getrlimit(resource, old_rlimits);
    if get_result.is_err() {
        debug!(
            "sys_rt_prlimit64: sys_rt_getrlimit failed: {:?}",
            get_result
        );
    }

    let set_result = sys_rt_setrlimit(resource, new_rlimits);
    if get_result.is_err() {
        debug!(
            "sys_rt_prlimit64: sys_rt_setrlimit executed, result: {:?}",
            set_result
        );
    }

    debug!("sys_rt_prlimit64: Completed. new_rlimits was NULL. Returning Ok(0).");
    Ok(0)
}
