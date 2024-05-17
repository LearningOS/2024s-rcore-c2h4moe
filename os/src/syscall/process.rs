//! Process management syscalls
use core::{mem::size_of, slice::from_raw_parts};

use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE}, loader::get_num_app, mm::{translated_byte_buffer, MapPermission}, sync::UPSafeCell, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_pid, suspend_current_and_run_next, task_mmap, task_munmap, TaskStatus
    }, timer::{get_time_ms, get_time_us}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
#[derive(Clone)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
}

impl TaskInfo {
    pub fn new() -> Self {
        TaskInfo {
            status: TaskStatus::Running,
            syscall_times: [0; MAX_SYSCALL_NUM],
            time: 0
        }
    }
}


lazy_static! {
    /// the task info of all apps
    pub static ref TASK_INFO: UPSafeCell<Vec<TaskInfo>> = {
        let mut info = Vec::new();
        let n = get_num_app();
        for _ in 0..n {
            info.push(TaskInfo::new());
        }
        unsafe{
            UPSafeCell::new(info)
        }
    };
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
    let token = current_user_token();
    let pspace = translated_byte_buffer(token, _ts as *const u8, size_of::<TimeVal>());
    let t = get_time_us();
    
    let info = TimeVal{
        sec: t / 1000000,
        usec: t
    };
    let info_slice = unsafe{
        from_raw_parts(&info as *const TimeVal as *const u8, size_of::<TimeVal>())
    };
    let mut start = 0;
    for dst in pspace {
        let src = &info_slice[start..start + dst.len()];
        dst.copy_from_slice(src);
        start += dst.len();
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    let pid = get_current_pid();
    let token = current_user_token();
    let pspace = translated_byte_buffer(token, _ti as *const u8, size_of::<TaskInfo>());
    let mut info = TASK_INFO.exclusive_access()[pid].clone();
    info.time = get_time_ms() - info.time;
    let data;
    unsafe{
        data = from_raw_parts(&info as *const TaskInfo as usize as *const u8, size_of::<TaskInfo>());
    }
    let mut start = 0;
    for dst in pspace {
        let src = &data[start..start + dst.len()];
        dst.copy_from_slice(src);
        start += dst.len();
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    if (_start & (PAGE_SIZE - 1)) != 0 || (_port & (!7)) != 0 || (_port & 7) == 0 {
        -1
    } else {
        let mut perm = MapPermission::U;
        if (_port & 1) != 0{
            perm = perm.union(MapPermission::R);
        }
        if (_port & 2) != 0{
            perm = perm.union(MapPermission::W);
        }
        if (_port & 4) != 0{
            perm = perm.union(MapPermission::X);
        }
        task_mmap(_start, _len, perm)
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    if (_start & (PAGE_SIZE - 1)) != 0 {
        -1
    } else {
        task_munmap(_start, _len)
    }
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
