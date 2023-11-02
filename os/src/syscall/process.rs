//! Process management syscalls
use alloc::sync::Arc;

use core::mem::size_of;


use crate::mm::{modify_byte_buffer, VirtAddr, check_none_map, MapPermission, check_exist_none_map};
use crate::task::{current_user_token, current_syscall_times, current_start_time};

use crate::timer::{get_time_ms, get_time_us};
use crate::{
    config::MAX_SYSCALL_NUM,
    loader::get_app_data_by_name,
    mm::{translated_refmut, translated_str},
    task::{
        add_task, current_task, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus,
    },
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        trace!("child pid {}", child.getpid());
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_get_time",
        current_task().unwrap().pid.0
    );
    let us = get_time_us();
    let ts = TimeVal{
        sec: us / 1_000_000,
        usec: us % 1_000_000
    };
    let _pts: *const TimeVal = &ts;
    let pts = _pts as *const u8;
    modify_byte_buffer(current_user_token(), _ts as *const u8, size_of::<TimeVal>(), pts);
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!(
        "kernel:pid[{}] sys_task_info",
        current_task().unwrap().pid.0
    );
    let time = get_time_ms() - current_start_time();
    trace!("kernel: task time from 1st use: {}ms", time);
    let __ti = TaskInfo {
        status: TaskStatus::Running,
        syscall_times: current_syscall_times(),
        time,
    };
    let _pti: *const TaskInfo = &__ti;
    let pti = _pti as *const u8;
    modify_byte_buffer(current_user_token(), _ti as *const u8, size_of::<TaskInfo>(), pti);
    0
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_mmap",
        current_task().unwrap().pid.0
    );
    if _port & !0x7 != 0 {
        error!("sys_mmap port format wrong!");
        return -1;
    } else if _port & 0x7 == 0 {
        error!("sys_mmap port meaningless!");
        return -1;
    } else if !VirtAddr::from(_start).aligned() {
        error!("sys_mmap start pa do not align!");
        return -1;
    }else if !check_none_map(current_user_token(), _start, _len) {
        error!("sys_mmap start to end pa exist map!");
        return -1;
    }
    let current_task = current_task().unwrap();
    current_task.pg_mmap(VirtAddr::from(_start), VirtAddr::from(_start+_len), MapPermission::from_bits_truncate((_port as u8) << 1) | MapPermission::U);
    0
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_munmap",
        current_task().unwrap().pid.0
    );
    if !VirtAddr::from(_start).aligned() {
        error!("sys_munmap start pa do not align!");
        return -1;
    }else if !check_exist_none_map(current_user_token(), _start, _len) {
        error!("sys_munmap start to end pa exist None map!");
        return -1;
    }
    let current_task = current_task().unwrap();
    current_task.pg_munmap(VirtAddr::from(_start), VirtAddr::from(_start+_len));
    0
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(_path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let path = translated_str(token, _path);
    if let Some(elf_data) = get_app_data_by_name(path.as_str()) {
        let current_task = current_task().unwrap();
        let spawn_task = current_task.spawn(elf_data);
        let spawn_id = spawn_task.pid.0 as isize;
        trace!(
            "kernel:pid[{}] spawn",
            spawn_id
        );
        add_task(spawn_task);
        spawn_id
    } else {
        -1
    }
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(_prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority",
        current_task().unwrap().pid.0
    );
    if _prio <= 1 {
        return -1;
    }
    let current_task = current_task().unwrap();
    current_task.set_current_priority(_prio as usize);
    current_task.get_priority_number() as isize
}
