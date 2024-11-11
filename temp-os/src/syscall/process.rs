//! Process management syscalls

use core::ptr::addr_of;
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,
    },
};
use crate::mm::translated_byte_buffer;
use crate::task::{current_user_token, get_syscall_times, get_task_time, mmap_cur_task, munmap_cur_task};
use crate::timer::get_time_us;

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
impl TaskInfo {
    fn get_task_info()->Self{
        TaskInfo{
            status: TaskStatus::Running,
            syscall_times: get_syscall_times(),
            time: get_task_time(),
        }
    }
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
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let buffers =translated_byte_buffer(current_user_token(), ts as *const u8, core::mem::size_of::<TimeVal>());
    let us = get_time_us();
    let tv= TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    let mut tv_ptr =addr_of!(tv) as *const u8;
    for buffer in buffers{
        unsafe {
            tv_ptr.copy_to(buffer.as_mut_ptr(),buffer.len());
            tv_ptr=tv_ptr.offset(buffer.len() as isize);
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let buffers =translated_byte_buffer(current_user_token(), ti as *const u8, core::mem::size_of::<TaskInfo>());
    let ti_temp=TaskInfo::get_task_info();
    let mut ti_temp_ptr=addr_of!(ti_temp) as *const u8;
    for buffer in buffers {
        unsafe {
            ti_temp_ptr.copy_to(buffer.as_mut_ptr(),buffer.len());
            ti_temp_ptr=ti_temp_ptr.offset(buffer.len() as isize);
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    match mmap_cur_task(start,len,port) {
        Ok(_) => 0,
        Err(e) => {error!("{e}");-1}
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    match munmap_cur_task(start,len) {
        Ok(_) => 0,
        Err(e) => {error!("{e}");-1}
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