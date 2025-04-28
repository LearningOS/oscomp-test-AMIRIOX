use core::ffi::{c_int, c_void};

use axerrno::LinuxResult;

use crate::ptr::{PtrWrapper, UserConstPtr, UserPtr};

use arceos_posix_api as api;
use arceos_posix_api::ctypes::rlimit;

pub fn sys_rt_sigprocmask(
    _how: i32,
    _set: UserConstPtr<c_void>,
    _oldset: UserPtr<c_void>,
    _sigsetsize: usize,
) -> LinuxResult<isize> {
    warn!("sys_rt_sigprocmask: not implemented");
    Ok(0)
}

pub fn sys_rt_sigaction(
    _signum: i32,
    _act: UserConstPtr<c_void>,
    _oldact: UserPtr<c_void>,
    _sigsetsize: usize,
) -> LinuxResult<isize> {
    warn!("sys_rt_sigaction: not implemented");
    Ok(0)
}

pub fn sys_rt_kill() -> LinuxResult<isize> {
    warn!("sys_rt_kill(beg): don't kill me, please.");
    Ok(0)
}

pub fn sys_rt_sigtimedwait() -> LinuxResult<isize> {
    warn!("sys_rt_sigtimedwait: I'm always waiting for you.");
    Ok(0)
}

pub fn sys_rt_getrlimit(resource: c_int, rlimits: UserPtr<rlimit>) -> LinuxResult<isize> {
    info!("sys_rt_getrlimit: 天空即为极限!");
    Ok(unsafe { api::sys_getrlimit(resource, rlimits.get()?).try_into().unwrap() })
}

pub fn sys_rt_setrlimit(resource: c_int, rlimits: UserPtr<rlimit>) -> LinuxResult<isize> {
    info!("sys_rt_getrlimit: 天空即为极限!");
    Ok(unsafe { api::sys_setrlimit(resource, rlimits.get()?).try_into().unwrap() })
}
