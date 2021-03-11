use crate::task::{
    suspend_current_and_run_next,
    exit_current_and_run_next,
    get_current_task_id,
    set_current_priority,
    MIN_PRIORITY,
};
use crate::loader::valid_app_buf;
use crate::timer::{TimeVal, get_time_val};

pub fn sys_exit(_exit_code: i32) -> ! {
    //println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_set_priority(priority: isize) -> isize {
    if priority < MIN_PRIORITY {
        -1
    } else {
        set_current_priority(priority);
        priority
    }
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time(buf: usize, _tz: usize) -> isize {
    let app_id = get_current_task_id();
    if !valid_app_buf(
        app_id,
        buf,
        core::mem::size_of::<TimeVal>(),
    ) {
        -1
    } else {
        let time = get_time_val();
        unsafe {
            (buf as *mut TimeVal).write_volatile(time);
        }
        0
    }
}
