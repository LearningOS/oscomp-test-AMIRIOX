use axtask::{TaskExtRef, exit};
use super::signal::{send_signal_thread, Signal};

pub fn do_exit(exit_code: i32, group_exit: bool) -> ! {
    let curr = axtask::current();
    let thr = &curr.task_ext().thread;

    let clear_child_pid = curr.task_ext().thread_data().clear_child_tid() as *mut i32;
    if !clear_child_pid.is_null() {
        // SAFETY: Not safe
        unsafe { *clear_child_pid = 0 }
    }
    let proc = thr.process();
    if thr.exit(exit_code) {
        proc.exit();
    }

    if group_exit {
        proc.group_exit();
        for thrs in proc.threads() {
            send_signal_thread(thrs.tid() as core::ffi::c_int, Signal::SIGKILL as u32 as i32); 
        }
    }
    axtask::exit(exit_code);
}
