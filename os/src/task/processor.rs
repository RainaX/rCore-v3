use super::TaskControlBlock;
use alloc::sync::Arc;
use core::cell::RefCell;
use lazy_static::*;
use super::{fetch_task, TaskStatus};
use super::__switch;
use crate::trap::TrapContext;
use crate::mm::{VirtAddr, MapPermission};

pub struct Processor {
    inner: RefCell<ProcessorInner>,
}

unsafe impl Sync for Processor {}

struct ProcessorInner {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx_ptr: usize,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(ProcessorInner {
                current: None,
                idle_task_cx_ptr: 0,
            }),
        }
    }

    fn get_idle_task_cx_ptr2(&self) -> *const usize {
        let inner = self.inner.borrow();
        &inner.idle_task_cx_ptr as *const usize
    }

    pub fn run(&self) {
        loop {
            if let Some(task) = fetch_task() {
                let idle_task_cx_ptr2 = self.get_idle_task_cx_ptr2();

                let mut task_inner = task.acquire_inner_lock();
                let next_task_cx_ptr2 = task_inner.get_task_cx_ptr2();
                task_inner.task_status = TaskStatus::Running;
                drop(task_inner);

                self.inner.borrow_mut().current = Some(task);
                unsafe {
                    __switch(
                        idle_task_cx_ptr2,
                        next_task_cx_ptr2,
                    );
                }
            }
        }
    }

    pub fn take_current(&self) -> Option<Arc<TaskControlBlock>> {
        self.inner.borrow_mut().current.take()
    }

    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.inner.borrow().current.as_ref().map(|task| Arc::clone(task))
    }
}


lazy_static! {
    pub static ref PROCESSOR: Processor = Processor::new();
}

pub fn run_tasks() {
    PROCESSOR.run();
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.current()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.acquire_inner_lock().get_user_token();
    token
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task().unwrap().acquire_inner_lock().get_trap_cx()
}

pub fn mmap_current(start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission) -> Result<(), ()> {
    let task = current_task().unwrap();
    let result = task.acquire_inner_lock().insert_framed_area(start_va, end_va, permission);
    result
}

pub fn munmap_current(start_va: VirtAddr, end_va: VirtAddr) {
    let task = current_task().unwrap();
    task.acquire_inner_lock().unmap_framed_area(start_va, end_va);
}

pub fn set_current_priority(priority: isize) {
    let task = current_task().unwrap();
    task.acquire_inner_lock().set_priority(priority);
}

pub fn schedule(switched_task_cx_ptr2: *const usize) {
    let idle_task_cx_ptr2 = PROCESSOR.get_idle_task_cx_ptr2();
    unsafe {
        __switch(
            switched_task_cx_ptr2,
            idle_task_cx_ptr2,
        );
    }
}
