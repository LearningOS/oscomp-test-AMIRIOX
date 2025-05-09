use super::exit::do_exit;
use super::task::{ProcessData, ThreadData};
use axerrno::{LinuxError, LinuxResult};
use axhal::arch::TrapFrame;
use axhal::trap::{POST_TRAP, register_trap_handler};
use axtask::{TaskExtRef, current};
use core::{
    cmp::{Eq, PartialEq},
    ffi::c_int,
    fmt::Result,
    marker::Sized,
};

#[macro_export]
macro_rules! define_signals {
    (
        #[repr(u32)]
        $(#[$enum_meta:meta])*
        $vis:vis enum $Enum_name:ident {
            $(
                $(#[$field_meta:meta])*
                $FIELD:ident = $value:expr
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $Enum_name {
            $($FIELD = $value),*
        }

        impl $Enum_name {
            pub fn from_u32(sig: c_int) -> Option<Self> {
                match sig {
                    $($value => Some(Self::$FIELD),)*
                    _ => None,
                }
            }
        }

        bitflags::bitflags! {
            $(#[$enum_meta])*
            pub struct SigMask: u32 {
                $(
                    const $FIELD = 1 << $value;
                )*
            }
        }
    };
}

/// Signal types
/// See `man 7 signal` (https://man.archlinux.org/man/signal.7.en)
/// See also  /usr/include/asm-generic/signal.h
define_signals! {
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Signal {
        /// Term    Hangup detected on controlling terminal or death of controlling process
        SIGHUP = 1,
        /// Term    Interrupt from keyboard
        SIGINT = 2,
        /// Core    Quit from keyboard
        SIGQUIT = 3,
        /// Core    Illegal Instruction
        SIGILL = 4,
        /// Core    Trace/breakpoint trap
        SIGTRAP = 5,
        /// Core    Abort signal from abort(3)
        SIGABRT = 6,
        /*
        /// Core    IOT trap. A synonym for SIGABRT
        SIGIOT = 6,
        */
        /// Core    Bus error (bad memory access)
        SIGBUS = 7,
        /// Core    Erroneous arithmetic operation
        SIGFPE = 8,
        /// Term    Kill signal
        SIGKILL = 9,
        /// Term    User-defined signal 1
        SIGUSR1 = 10,
        /// Core    Invalid memory reference
        SIGSEGV = 11,
        /// Term    User-defined signal 2
        SIGUSR2 = 12,
        /// Term    Broken pipe: write to pipe with no readers; see pipe(7)
        SIGPIPE = 13,
        /// Term    Timer signal from alarm(2)
        SIGALRM = 14,
        /// Term    Termination signal
        SIGTERM = 15,
        /// Term    Stack fault on coprocessor (unused)
        SIGSTKFLT = 16,
        /// Ign     Child stopped or terminated
        SIGCHLD = 17,
        /// Cont    Continue if stopped
        SIGCONT = 18,
        /// Stop    Stop process
        SIGSTOP = 19,
        /// Stop    Stop typed at terminal
        SIGTSTP = 20,
        /// Stop    Terminal input for background process
        SIGTTIN = 21,
        /// Stop    Terminal output for background process
        SIGTTOU = 22,
        /// Ign     Urgent condition on socket (4.2BSD)
        SIGURG = 23,
        /// Core    CPU time limit exceeded (4.2BSD); see setrlimit(2)
        SIGXCPU = 24,
        /// Core    File size limit exceeded (4.2BSD); see setrlimit(2)
        SIGXFSZ = 25,
        /// Term    Virtual alarm clock (4.2BSD)
        SIGVTALRM = 26,
        /// Term    Profiling timer expired
        SIGPROF = 27,
        /// Ign     Window resize signal (4.3BSD, Sun)
        SIGWINCH = 28,
        /// Term    I/O now possible (4.2BSD)
        SIGIO = 29,
        /*
        /// Term    Pollable event (Sys V); synonym for SIGIO
        SIGPOLL = 29,
        */
        /// Term    Power failure (System V)
        SIGPWR = 30,
        /// Core    Bad system call (SVr4); see also seccomp(2)
        SIGSYS = 31,
        /*
        /// Core    Synonymous with SIGSYS
        SIGUNUSED = 31,
        */
    }
}

/// Count of signals
const _NSIG: i32 = 32;
/// Real-time signals (platform-specific)
pub const SIGRTMIN: i32 = 32;
/// Maximum real-time signal (platform-specific)
pub const SIGRTMAX: i32 = _NSIG - 1;

#[derive(Default)]
pub enum SigDisposition {
    #[default]
    Default,
    Ignore,
    CoreDump,
    Terminate,
    Stop,
    Continue,
}

#[derive(Default)]
pub struct SignalAction {
    // TODO
    pub disposition: SigDisposition,
}

#[derive(Clone, Copy)]
pub enum SignalOSAction {
    CoreDump,
    Terminate,
    Stop,
    Continue,
    Handler { add_blocked: SigMask },
}

const DEFAULT_ACTIONS: [SigDisposition; 32] = [
    // Unspecified
    SigDisposition::Ignore,
    // SIGHUP
    SigDisposition::Terminate,
    // SIGINT
    SigDisposition::Terminate,
    // SIGQUIT
    SigDisposition::CoreDump,
    // SIGILL
    SigDisposition::CoreDump,
    // SIGTRAP
    SigDisposition::CoreDump,
    // SIGABRT
    SigDisposition::CoreDump,
    // SIGBUS
    SigDisposition::CoreDump,
    // SIGFPE
    SigDisposition::CoreDump,
    // SIGKILL
    SigDisposition::Terminate,
    // SIGUSR1
    SigDisposition::Terminate,
    // SIGSEGV
    SigDisposition::CoreDump,
    // SIGUSR2
    SigDisposition::Terminate,
    // SIGPIPE
    SigDisposition::Terminate,
    // SIGALRM
    SigDisposition::Terminate,
    // SIGTERM
    SigDisposition::Terminate,
    // SIGSTKFLT
    SigDisposition::Terminate,
    // SIGCHLD
    SigDisposition::Ignore,
    // SIGCONT
    SigDisposition::Continue,
    // SIGSTOP
    SigDisposition::Stop,
    // SIGTSTP
    SigDisposition::Stop,
    // SIGTTIN
    SigDisposition::Stop,
    // SIGTTOU
    SigDisposition::Stop,
    // SIGURG
    SigDisposition::Ignore,
    // SIGXCPU
    SigDisposition::CoreDump,
    // SIGXFSZ
    SigDisposition::CoreDump,
    // SIGVTALRM
    SigDisposition::Terminate,
    // SIGPROF
    SigDisposition::Terminate,
    // SIGWINCH
    SigDisposition::Ignore,
    // SIGIO
    SigDisposition::Terminate,
    // SIGPWR
    SigDisposition::Terminate,
    // SIGSYS
    SigDisposition::CoreDump,
];

/// Find proc by pid
/// Find qualified thread belonging to the proc to recv sig
/// Add sig to `pending`
pub fn send_signal_proc(pid: c_int, sig: c_int) -> LinuxResult<isize> {
    let cur_proc = super::task::PROCESS_TABLE
        .read()
        .get(&(pid as u32))
        .ok_or(LinuxError::ESRCH)?;

    if sig == 0 {
        return Ok(0);
    }

    let signal_index = SigMask::from_bits(1 << sig).ok_or(LinuxError::EINVAL)?;
    let signal = Signal::from_u32(sig).ok_or(LinuxError::EINVAL)?;

    for thread in cur_proc.threads().iter() {
        let thread_data: &ThreadData = thread.data().unwrap();
        if !thread_data.blocked.contains(signal_index) {
            // Checked by SigMask
            thread_data.pending.lock().push_back(signal);
            return Ok(0);
        }
    }
    let proc_data: &ProcessData = cur_proc.data().unwrap();
    proc_data.shared.lock().push_back(signal);
    Ok(0)
}

pub fn send_signal_thread(tid: c_int, sig: c_int) -> LinuxResult<isize> {
    let thread = super::task::THREAD_TABLE
        .read()
        .get(&(tid as u32))
        .ok_or(LinuxError::ESRCH)?;
    let thread_data: &ThreadData = thread.data().unwrap();

    let signal_index = SigMask::from_bits(1 << sig).ok_or(LinuxError::EINVAL)?;
    let signal = Signal::from_u32(sig).ok_or(LinuxError::EINVAL)?;

    if !thread_data.blocked.contains(signal_index) {
        thread_data.pending.lock().push_back(signal);
        Ok(0)
    } else {
        Err(LinuxError::EINVAL)
    }
}

pub fn handle_signal(on_action: &SignalAction, signo: u32) -> Option<SignalOSAction> {
    match on_action.disposition {
        SigDisposition::Default => match DEFAULT_ACTIONS[signo as usize] {
            SigDisposition::Ignore => None,
            SigDisposition::Default => panic!("Invalid default disposition"),
            SigDisposition::Stop => Some(SignalOSAction::Stop),
            SigDisposition::Continue => Some(SignalOSAction::Continue),
            SigDisposition::Terminate => Some(SignalOSAction::Terminate),
            SigDisposition::CoreDump => Some(SignalOSAction::CoreDump),
        },
        SigDisposition::CoreDump => Some(SignalOSAction::CoreDump),
        SigDisposition::Terminate => Some(SignalOSAction::Terminate),
        SigDisposition::Stop => Some(SignalOSAction::Stop),
        SigDisposition::Continue => Some(SignalOSAction::Continue),
        SigDisposition::Ignore => None,
    }
}
/*
pending 存放信号, 由 send_signal 发送, 顺便快速检查有无能解锁的任务

内核态返回前 check_signals, 从 pending 取一个没 blocked 的信号,
通过 id 从进程信号行为数组[1]中取出对应的 SigAction
通过 handle_signal 把 SigAction 根据其包含的 SigDisposition 映射到 SigOSAction
模式匹配 SigOSAction 来执行对应操作

[1]: 信号行为数组有 Default 值, 所以在没有 rt_sigaction[2] 的情况下模式匹配会到 Disposition::Default 处理, 提供了 DEFUALT_ACTIONS
[2]: 获取信号操作或修改信号操作, 实际上是获取或更改 ProcessData 的 signal_actions
*/
pub fn check_signals(tf: &mut TrapFrame) -> bool {
    info!("Handle signals.");
    let current = axtask::current();
    let data = current.task_ext().thread_data();
    let actions = current.task_ext().process_data().actions.lock();

    let mask = !data.blocked;
    let (signo, on_action) = loop {
        let pending = data.pending.lock();
        let Some(sig) = pending.front() else {
            return false;
        };
        let signo = *sig as u32;
        if mask.contains(SigMask::from_bits(1 << signo).expect("Wrong signo")) {
            continue;
        }
        if let Some(on_action) = handle_signal(&actions[signo as usize], signo) {
            break (signo, on_action);
        }
    };
    drop(actions);
    match on_action {
        SignalOSAction::CoreDump => {
            do_exit(128 + signo as i32, true);
        }
        SignalOSAction::Terminate => {
            do_exit(128 + signo as i32, true);
        }
        SignalOSAction::Stop => {
            // TODO
            do_exit(1, true);
        }
        SignalOSAction::Continue => {
            // TODO: continue
        }
        SignalOSAction::Handler { add_blocked } => {
            // TODO: add blocked
        }
    }
    unimplemented!("😅: check_signals");
}

#[register_trap_handler(POST_TRAP)]
fn post_trap_callback(tf: &mut TrapFrame, from_user: bool) {
    if !from_user {
        return;
    }
    check_signals(tf);
}
