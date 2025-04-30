use core::ffi::{CStr, c_char, c_void};

use arceos_posix_api::{self as api, ctypes::mode_t};
use axerrno::LinuxResult;

use crate::ptr::{PtrWrapper, UserConstPtr, UserPtr};

pub fn sys_read(fd: i32, buf: UserPtr<c_void>, count: usize) -> LinuxResult<isize> {
    let buf = buf.get_as_bytes(count)?;
    Ok(api::sys_read(fd, buf, count))
}

pub fn sys_write(fd: i32, buf: UserConstPtr<c_void>, count: usize) -> LinuxResult<isize> {
    let buf = buf.get_as_bytes(count)?;
    Ok(api::sys_write(fd, buf, count))
}

pub fn sys_writev(
    fd: i32,
    iov: UserConstPtr<api::ctypes::iovec>,
    iocnt: i32,
) -> LinuxResult<isize> {
    let iov = iov.get_as_bytes(iocnt as _)?;
    unsafe { Ok(api::sys_writev(fd, iov, iocnt)) }
}

pub fn sys_openat(
    dirfd: i32,
    path: UserConstPtr<c_char>,
    flags: i32,
    modes: mode_t,
) -> LinuxResult<isize> {
    let path = path.get_as_null_terminated()?;
    // ax_println!("create {:?}", unsafe { CStr::from_ptr(path.as_ptr()) });
    Ok(api::sys_openat(dirfd, path.as_ptr(), flags, modes) as _)
}

pub fn sys_open(path: UserConstPtr<c_char>, flags: i32, modes: mode_t) -> LinuxResult<isize> {
    use arceos_posix_api::AT_FDCWD;
    sys_openat(AT_FDCWD as _, path, flags, modes)
}

pub fn sys_unlink(pathname: UserConstPtr<c_char>) -> LinuxResult<isize> {
    let path_name = pathname.get_as_str()?;
    //ax_println!("{}", path_name);
    let (dir_prefix, file_name) = path_name.rsplit_once('/').unwrap();

    //ax_println!("axfs::fops::Directory::open({})", dir_prefix);
    let dir =
        axfs::fops::Directory::open_dir(dir_prefix, &axfs::fops::OpenOptions::new().set_read(true))
            .unwrap();
    dir.remove_file(file_name);
    // ax_println!("Please don't go💔");
    Ok(0)
}
