use crate::mm::{MemorySet, PhysPageNum, KERNEL_SPACE, VirtAddr, MapPermission};
use crate::trap::{TrapContext, trap_handler};
use crate::config::{TRAP_CONTEXT};
use super::TaskContext;
use super::{PidHandle, pid_alloc, KernelStack};
use super::stride_scheduler::SchedBlock;
use alloc::sync::{Weak, Arc};
use alloc::vec;
use alloc::vec::Vec;
use spin::{Mutex, MutexGuard};
use crate::fs::{File, Stdin, Stdout};

pub struct TaskControlBlock {
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    inner: Mutex<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub task_cx_ptr: usize,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
    pub sched_block: Option<SchedBlock>,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
}


impl TaskControlBlockInner {
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize
    }

    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }

    pub fn insert_framed_area(&mut self, start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission) -> Result<(), ()> {
        self.memory_set.insert_framed_area(start_va, end_va, permission)
    }

    pub fn unmap_framed_area(&mut self, start_va: VirtAddr, end_va: VirtAddr) {
        self.memory_set.unmap_framed_area(start_va, end_va);
    }

    pub fn set_priority(&mut self, priority: isize) {
        self.sched_block.as_mut().unwrap().set_priority(priority);
    }

    fn get_status(&self) -> TaskStatus {
        self.task_status
    }

    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }

    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len())
            .find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
}

impl TaskControlBlock {
    pub fn acquire_inner_lock(&self) -> MutexGuard<TaskControlBlockInner> {
        self.inner.lock()
    }

    pub fn new(elf_data: &[u8]) -> Option<Self> {
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data)?;
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle)?;
        let kernel_stack_top = kernel_stack.get_top();
        let task_cx_ptr = kernel_stack.push_on_top(TaskContext::goto_trap_return());
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: Mutex::new(TaskControlBlockInner {
                trap_cx_ppn,
                base_size: user_sp,
                task_cx_ptr: task_cx_ptr as usize,
                task_status: TaskStatus::Ready,
                memory_set,
                parent: None,
                children: Vec::new(),
                exit_code: 0,
                sched_block: None,
                fd_table: vec![
                    Some(Arc::new(Stdin)),
                    Some(Arc::new(Stdout)),
                    Some(Arc::new(Stdout)),
                ],
            }),
        };

        let trap_cx = task_control_block.acquire_inner_lock().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        Some(task_control_block)
    }

    pub fn exec(&self, elf_data: &[u8]) -> Result<(), ()> {
        let (memory_set, user_sp, entry_point) = match MemorySet::from_elf(elf_data) {
            Some(x) => x,
            None => return Err(()),
        };
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        let mut inner = self.acquire_inner_lock();
        inner.memory_set = memory_set;
        inner.trap_cx_ppn = trap_cx_ppn;
        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );

        Ok(())
    }

    pub fn fork(self: &Arc<TaskControlBlock>) -> Option<Arc<TaskControlBlock>> {
        let mut parent_inner = self.acquire_inner_lock();
        let memory_set = MemorySet::from_existed_user(
            &parent_inner.memory_set
        )?;
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle)?;
        let kernel_stack_top = kernel_stack.get_top();
        let task_cx_ptr = kernel_stack.push_on_top(TaskContext::goto_trap_return());
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent_inner.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: Mutex::new(TaskControlBlockInner {
                trap_cx_ppn,
                base_size: parent_inner.base_size,
                task_cx_ptr: task_cx_ptr as usize,
                task_status: TaskStatus::Ready,
                memory_set,
                parent: Some(Arc::downgrade(self)),
                children: Vec::new(),
                exit_code: 0,
                sched_block: None,
                fd_table: new_fd_table,
            }),
        });

        parent_inner.children.push(task_control_block.clone());
        
        let trap_cx = task_control_block.acquire_inner_lock().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        Some(task_control_block)
    }

    pub fn spawn(self: &Arc<TaskControlBlock>, elf_data: &[u8]) -> Option<Arc<TaskControlBlock>> {
        let task_control_block = Arc::new(TaskControlBlock::new(elf_data)?);
        let mut parent_inner = self.acquire_inner_lock();
        parent_inner.children.push(task_control_block.clone());
        task_control_block.acquire_inner_lock().parent = Some(Arc::downgrade(self));
        Some(task_control_block)
    }


    pub fn getpid(&self) -> usize {
        self.pid.0
    }
}


#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}
