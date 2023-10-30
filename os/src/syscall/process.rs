//! Process management syscalls
use core::mem::size_of;

use alloc::string::ToString;

use crate::mm::{modify_byte_buffer, VirtAddr, check_none_map, MapPermission, check_exist_none_map};
// use crate::mm::{modify_byte_buffer};
use crate::task::{current_user_token, TASK_MANAGER};

use crate::timer::get_time_ms;
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,
    }, timer::get_time_us,
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
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
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
    trace!("kernel: sys_task_info");
    trace!("kernel: now running task id: {}", TASK_MANAGER.get_current_task().to_string());
    let time = get_time_ms() - TASK_MANAGER.get_current_start_time();
    trace!("kernel: task time from 1st use: {}ms", time);
    let __ti = TaskInfo {
        status: TaskStatus::Running,
        syscall_times: TASK_MANAGER.syscall_times_clone(),
        time,
    };
    let _pti: *const TaskInfo = &__ti;
    let pti = _pti as *const u8;
    modify_byte_buffer(current_user_token(), _ti as *const u8, size_of::<TaskInfo>(), pti);
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
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
    error!("------");
    TASK_MANAGER.pg_mmap(VirtAddr::from(_start), VirtAddr::from(_start+_len), MapPermission::from_bits_truncate((_port as u8) << 1) | MapPermission::U);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if !VirtAddr::from(_start).aligned() {
        error!("sys_munmap start pa do not align!");
        return -1;
    }else if !check_exist_none_map(current_user_token(), _start, _len) {
        error!("sys_munmap start to end pa exist None map!");
        return -1;
    }
    TASK_MANAGER.pg_munmap(VirtAddr::from(_start), VirtAddr::from(_start+_len));
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
