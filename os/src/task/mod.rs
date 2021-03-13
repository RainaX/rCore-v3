mod binary_heap;
mod context;
mod stride_scheduler;
mod switch;
mod task;

use crate::loader::{get_num_app, get_app_data};
use crate::trap::TrapContext;
use core::cell::RefCell;
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use alloc::vec::Vec;
use crate::mm::{VirtAddr, MapPermission};
use stride_scheduler::StrideScheduler;

pub use context::TaskContext;
pub use stride_scheduler::MIN_PRIORITY;

pub struct TaskManager {
    //num_app: usize,
    inner: RefCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,
    scheduler: StrideScheduler,
}

unsafe impl Sync for TaskManager {}


lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        println!("num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        let mut scheduler = StrideScheduler::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(
                get_app_data(i),
                i,
            ).unwrap());
            scheduler.init_sched_block(i);
        }
        
        TaskManager {
            //num_app,
            inner: RefCell::new(TaskManagerInner {
                tasks,
                current_task: 0,
                scheduler,
            }),
        }
    };
}


impl TaskManager {
    fn run_first_task(&self) {
        let mut inner = self.inner.borrow_mut();
        let first_block = inner.scheduler.get_next_sched_block().unwrap();
        inner.current_task = first_block.id;
        inner.tasks[first_block.id].task_status = TaskStatus::Running;
        inner.tasks[first_block.id].sched_block = Some(first_block);
        let next_task_cx_ptr2 = inner.tasks[first_block.id].get_task_cx_ptr2();
        core::mem::drop(inner);
        let _unused: usize = 0;
        unsafe {
            __switch(
                &_unused as *const _,
                next_task_cx_ptr2,
            );
        }
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
        let block = inner.tasks[current].sched_block.take().unwrap();
        inner.scheduler.add_sched_block(block);
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
        inner.tasks[current].sched_block = None;
    }

    fn find_next_task(&self) -> Option<usize> {
        let mut inner = self.inner.borrow_mut();
        match inner.scheduler.get_next_sched_block() {
            Some(block) => {
                inner.tasks[block.id].sched_block = Some(block);
                Some(block.id)
            },
            None => None,
        }
    }

    fn get_current_token(&self) -> usize {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        inner.tasks[current].get_user_token()
    }

    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        inner.tasks[current].get_trap_cx()
    }

    fn mmap_current(&self, start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission) -> Result<(), ()> {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].insert_framed_area(start_va, end_va, permission)
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr2 = inner.tasks[current].get_task_cx_ptr2();
            let next_task_cx_ptr2 = inner.tasks[next].get_task_cx_ptr2();
            core::mem::drop(inner);
            unsafe {
                __switch(
                    current_task_cx_ptr2,
                    next_task_cx_ptr2,
                );
            }
        } else {
            panic!("All applications completed!");
        }
    }
}


pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

pub fn mmap_current(start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission) -> Result<(), ()> {
    TASK_MANAGER.mmap_current(start_va, end_va, permission)
}
