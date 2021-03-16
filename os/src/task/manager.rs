use super::TaskControlBlock;
use super::stride_scheduler::StrideScheduler;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::Mutex;
use lazy_static::*;

pub struct TaskManager {
    map: BTreeMap<usize, Arc<TaskControlBlock>>,
    scheduler: StrideScheduler,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            scheduler: StrideScheduler::new(),
        }
    }

    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        let pid = task.pid.0;
        let mut inner = task.acquire_inner_lock();
        if inner.sched_block.is_none() {
            self.scheduler.init_sched_block(pid);
        } else {
            self.scheduler.add_sched_block(inner.sched_block.take().unwrap());
        }
        core::mem::drop(inner);
        self.map.insert(pid, task);
    }

    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let next_block = self.scheduler.get_next_sched_block()?;
        let next_task = self.map.remove(&next_block.id).unwrap();
        next_task.acquire_inner_lock().sched_block = Some(next_block);
        Some(next_task)
    }
}


lazy_static! {
    pub static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}


pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.lock().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.lock().fetch()
}

