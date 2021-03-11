mod context;
mod switch;
mod task;
mod binary_heap;
mod stride_scheduler;

use crate::config::{MAX_APP_NUM, CLOCK_FREQ, MAX_APP_SEC};
use crate::loader::{get_num_app, init_app_cx};
use crate::timer::get_time;
use core::cell::RefCell;
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use stride_scheduler::StrideScheduler;

pub use context::TaskContext;
pub use stride_scheduler::MIN_PRIORITY;

pub struct TaskManager {
    //num_app: usize,
    inner: RefCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
    scheduler: StrideScheduler,
}

unsafe impl Sync for TaskManager {}


lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [
            TaskControlBlock { task_cx_ptr: 0, task_status: TaskStatus::UnInit, sched_block: None, start_time: 0, cum_time: 0 };
            MAX_APP_NUM
        ];
        let mut scheduler = StrideScheduler::new();

        for i in 0..num_app {
            tasks[i].task_cx_ptr = init_app_cx(i) as *const _ as usize;
            tasks[i].task_status = TaskStatus::Ready;
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
        inner.tasks[first_block.id].start_time = get_time();

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

        // Calculate cumulated running time
        let start_time = inner.tasks[current].start_time;
        inner.tasks[current].cum_time += get_time() - start_time;
        inner.tasks[current].start_time = 0;

        // If cumulated running time for current task has exceeded the limit,
        // Stop current app immediately
        if inner.tasks[current].cum_time >= CLOCK_FREQ * MAX_APP_SEC {
            core::mem::drop(inner);
            self.mark_current_exited();
        } else {
            inner.tasks[current].task_status = TaskStatus::Ready;
            let block = inner.tasks[current].sched_block.take().unwrap();

            // Re-insert the schedule block to heap
            inner.scheduler.add_sched_block(block);
        }
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
                // Save the schedule block of current thread
                // which might be inserted into the heap again
                inner.tasks[block.id].sched_block = Some(block);
                Some(block.id)
            },
            None => None,
        }
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;

            // Save the latest start time to calculate cumulated running time
            inner.tasks[next].start_time = get_time();

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

    fn get_current_task_id(&self) -> usize {
        self.inner.borrow().current_task
    }

    fn set_current_priority(&self, priority: isize) {
        let current = self.get_current_task_id();
        self.inner.borrow_mut().tasks[current].sched_block.as_mut().unwrap().set_priority(priority);
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

pub fn get_current_task_id() -> usize {
    TASK_MANAGER.get_current_task_id()
}

pub fn set_current_priority(priority: isize) {
    TASK_MANAGER.set_current_priority(priority);
}
