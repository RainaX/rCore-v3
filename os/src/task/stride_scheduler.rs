use core::cmp::Ordering;
use super::binary_heap::BinaryHeap;

pub const MIN_PRIORITY: isize = 2;

const BIG_STRIDE: u64 = u64::MAX;
const INIT_PRIORITY: isize = 16;


pub struct StrideScheduler {
    heap: BinaryHeap<SchedBlock>,
}

#[derive(Copy, Clone)]
pub struct SchedBlock {
    pub id: usize,
    stride: Stride,
    pass: u64, 
}

#[derive(Copy, Clone)]
struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // If the difference between two values is greater than BIG_STRIDE/2, 
        // there should be an overflow, which means the smaller numeric
        // value has the greater stride.
        // Otherwise, the greater numeric value has the greater stride.

        if self.0 < other.0 {
            if other.0 - self.0 <= BIG_STRIDE / 2 {
                Some(Ordering::Less)
            } else {
                Some(Ordering::Greater)
            }
        } else if self.0 > other.0 {
            if self.0 - other.0 <= BIG_STRIDE / 2 {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Less)
            }
        } else {
            None
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}


impl PartialOrd for SchedBlock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.stride.partial_cmp(&other.stride)
    }
}

impl PartialEq for SchedBlock {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}


impl SchedBlock {
    pub fn set_priority(&mut self, priority: isize) {
        // The parameter should be checked in syscall module before invoking this function.
        // If it is still wrong here, just panic.
        if priority < MIN_PRIORITY {
            panic!("Invalid priority!");
        }
        self.pass = BIG_STRIDE / priority as u64;
    }
}



impl StrideScheduler {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    pub fn init_sched_block(&mut self, id: usize) {
        let block = SchedBlock {
            id,
            stride: Stride(0),
            pass: BIG_STRIDE / INIT_PRIORITY as u64,
        };
        self.heap.insert(block);
    }

    pub fn add_sched_block(&mut self, block: SchedBlock) {
        self.heap.insert(block);
    }

    pub fn get_next_sched_block(&mut self) -> Option<SchedBlock> {
        match self.heap.pop_min() {
            Some(mut block) => {
                // Add pass to stride, so that the block can be
                // scheduled correctly after being inserted
                // to heap next time
                block.stride.0 = block.stride.0.overflowing_add(block.pass).0;
                Some(block)
            },
            None => None,
        }
    }
}
