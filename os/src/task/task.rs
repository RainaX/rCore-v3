use super::stride_scheduler::SchedBlock;

pub struct TaskControlBlock {
    pub task_cx_ptr: usize,
    pub task_status: TaskStatus,
    pub sched_block: Option<SchedBlock>,
    pub start_time: usize,
    pub cum_time: usize,
}


impl TaskControlBlock {
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize
    }
}


#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
