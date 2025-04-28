use core::ffi::c_void;

use axerrno::LinuxResult;

use crate::ptr::{UserConstPtr, UserPtr};

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

pub fn sys_rt_getrlimit() -> LinuxResult<isize> {
    warn!("sys_rt_getrlimit: 天空即为极限!");
    Ok(0)
}
