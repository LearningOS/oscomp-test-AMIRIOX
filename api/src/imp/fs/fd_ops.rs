use core::ffi::c_int;
use core::ffi::c_long;

use arceos_posix_api as api;
use axerrno::LinuxResult;

use axtask::{TaskExtRef, current};
use api::FD_TABLE;

pub fn check_fd_limit() -> LinuxResult<isize> {
    let curr = axtask::current();
    let data = curr.task_ext().process_data();

    let rlim_cur = data.fd_limit.lock().rlim_cur;
    let fd_no: u64 = FD_TABLE.read().count().try_into().unwrap();
    if fd_no >= rlim_cur {
        warn!("fd no = {}, more than rlimit: {} / *", fd_no, rlim_cur);
        return Err(axerrno::LinuxError::EMFILE);
    } else {
        debug!("current rlimit: {} < current fd: {}", rlim_cur, fd_no);
    }
    Ok(0)
}

pub fn sys_dup(old_fd: c_int) -> LinuxResult<isize> {
    check_fd_limit()?;
    Ok(api::sys_dup(old_fd) as _)
}

pub fn sys_dup3(old_fd: c_int, new_fd: c_int) -> LinuxResult<isize> {
    check_fd_limit()?;
    Ok(api::sys_dup2(old_fd, new_fd) as _)
}

pub fn sys_close(fd: c_int) -> LinuxResult<isize> {
    Ok(api::sys_close(fd) as _)
}

pub fn sys_fcntl(fd: c_int, cmd: c_int, arg: usize) -> LinuxResult<isize> {
    Ok(api::sys_fcntl(fd, cmd, arg) as _)
}

pub fn sys_lseek(fd: c_int, offset: c_long, whence: i32) -> LinuxResult<isize> {
    Ok(api::sys_lseek(fd, offset, whence) as _)
}
