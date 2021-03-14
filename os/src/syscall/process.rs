use crate::task::{
    suspend_current_and_run_next,
    exit_current_and_run_next,
    current_user_token,
    set_current_priority,
    MIN_PRIORITY,
};
use crate::mm::{MapPermission, translated_mut_ref, is_mapped};
use crate::timer::{TimeVal, get_time_val};
use crate::config::PAGE_SIZE;

pub fn sys_exit(_exit_code: i32) -> ! {
    //println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_set_priority(priority: isize) -> isize {
    if priority < MIN_PRIORITY {
        -1
    } else {
        set_current_priority(priority);
        priority
    }
}


pub fn sys_get_time(buf: usize, _tz: usize) -> isize {
    let mut start = buf / PAGE_SIZE * PAGE_SIZE;
    let end = buf + core::mem::size_of::<TimeVal>();
    while start < end {
        
        if !is_mapped(current_user_token(), start, MapPermission::U | MapPermission::W) {
            return -1;
        }
        start += PAGE_SIZE;
    }
    let time = get_time_val();
    let phys_buf: &mut TimeVal = translated_mut_ref(current_user_token(), buf);
    *phys_buf = time;
    0
}
