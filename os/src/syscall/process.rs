use crate::task::{
    suspend_current_and_run_next,
    exit_current_and_run_next,
    current_task,
    current_user_token,
    set_current_priority,
    add_task,
    spawn,
    MIN_PRIORITY,
};
use crate::mm::{
    MapPermission,
    translated_str,
    translated_refmut,
    is_mapped,
};
use crate::timer::{TimeVal, get_time_val};
use crate::config::PAGE_SIZE;
use crate::loader::get_app_data_by_name;
use alloc::sync::Arc;

pub fn sys_exit(exit_code: i32) -> ! {
    //println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next(exit_code);
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
    let phys_buf: &mut TimeVal = translated_refmut(current_user_token(), buf as *mut TimeVal);
    *phys_buf = time;
    0
}


pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}


pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = match current_task.fork() {
        Some(task) => task,
        None => return -1,
    };
    let new_pid = new_task.pid.0;

    let trap_cx = new_task.acquire_inner_lock().get_trap_cx();
    trap_cx.x[10] = 0;

    add_task(new_task);
    new_pid as isize
}


pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = match translated_str(token, path) {
        Some(path) => path,
        None => return -1,
    };
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        match task.exec(data) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    } else {
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let mut start = exit_code_ptr as usize;
    let end = start + core::mem::size_of::<i32>();
    while start < end {
        if !is_mapped(current_user_token(), start, MapPermission::U | MapPermission::W) {
            return -3;
        }
        start += PAGE_SIZE;
    }

    let task = current_task().unwrap();

    let mut inner = task.acquire_inner_lock();
    if inner.children
        .iter()
        .find(|p| { pid == -1 || pid as usize == p.getpid() })
        .is_none() {
        return -1;
    }
    let pair = inner.children
        .iter()
        .enumerate()
        .find(|(_, p)| {
            p.acquire_inner_lock().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);

        assert_eq!(Arc::strong_count(&child), 1);
        
        let found_pid = child.getpid();
        let exit_code = child.acquire_inner_lock().exit_code;
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
}


pub fn sys_spawn(path: *const u8) -> isize {
    let token = current_user_token();
    let path = match translated_str(token, path) {
        Some(path) => path,
        None => return -1,
    };
    let new_task = match spawn(path.as_str()) {
        Some(task) => task,
        None => return -1,
    };
    let new_pid = new_task.pid.0;

    add_task(new_task);
    new_pid as isize
}
        
