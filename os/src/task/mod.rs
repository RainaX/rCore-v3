mod binary_heap;
mod context;
mod manager;
mod pid;
mod processor;
mod stride_scheduler;
mod switch;
mod task;

use crate::loader::{get_app_data_by_name};
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use alloc::sync::Arc;
use manager::fetch_task;
use lazy_static::*;

pub use context::TaskContext;
pub use processor::{
    run_tasks,
    current_task,
    current_user_token,
    current_trap_cx,
    take_current_task,
    mmap_current,
    munmap_current,
    set_current_priority,
    schedule,
};
pub use manager::add_task;
pub use pid::{PidHandle, pid_alloc, KernelStack};
pub use stride_scheduler::MIN_PRIORITY;

pub fn suspend_current_and_run_next() {
    let task = take_current_task().unwrap();

    let mut task_inner = task.acquire_inner_lock();
    let task_cx_ptr2 = task_inner.get_task_cx_ptr2();
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);

    add_task(task);
    schedule(task_cx_ptr2);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    let task = take_current_task().unwrap();
    
    let mut inner = task.acquire_inner_lock();
    inner.task_status = TaskStatus::Zombie;
    inner.exit_code = exit_code;

    {
        let mut initproc_inner = INITPROC.acquire_inner_lock();
        for child in inner.children.iter() {
            child.acquire_inner_lock().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }

    inner.children.clear();
    
    inner.memory_set.recycle_data_pages();
    drop(inner);

    drop(task);

    let _unused: usize = 0;
    schedule(&_unused as *const _);
}

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(
        TaskControlBlock::new(get_app_data_by_name("initproc").unwrap()).unwrap()
    );
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}
