use core::cmp::Ordering;
use crate::config::MAX_APP_NUM;

pub struct BinaryHeap<T: PartialOrd + Copy> {
    data: [Option<T>; MAX_APP_NUM],
    size: usize,
}


impl<T: PartialOrd + Copy> BinaryHeap<T> {
    fn swap(&mut self, i: usize, j: usize) {
        let temp = self.data[i].take();
        self.data[i] = self.data[j].take();
        self.data[j] = temp;
    }

    pub fn new() -> Self {
        Self {
            data: [None; MAX_APP_NUM],
            size: 0,
        }
    }

    pub fn insert(&mut self, ele: T) {
        if self.size == MAX_APP_NUM {
            panic!("The binary heap is full!");
        }

        // insert the new element to the tail of array
        let mut cur = self.size;
        self.data[cur] = Some(ele);
        self.size += 1;

        // swap if current element is smaller than its parent
        while cur > 0 {
            let p = parent(cur);
            if let Some(Ordering::Less) = self.data[cur].as_ref().unwrap().partial_cmp(self.data[p].as_ref().unwrap()) {
                self.swap(cur, p);
                cur = p;
            } else {
                break;
            }
        }
    }

    pub fn pop_min(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }

        // Remove the element at head
        // Then move the tail element to head position
        let result = self.data[0].take();
        self.size -= 1;
        self.data[0] = self.data[self.size].take();

        // Swap until current element is not greater than both children
        if self.size > 0 {
            let mut cur = 0;
            loop {
                let mut next = cur;

                let l = lchild(cur);
                let r = rchild(cur);

                if l < self.size && r < self.size {
                    next = match self.data[l].as_ref().unwrap().partial_cmp(self.data[r].as_ref().unwrap()) {
                        Some(Ordering::Less) => l,
                        Some(Ordering::Greater) => r,
                        _ => l,
                    };
                } else if l < self.size {
                    next = l;
                }

                if let Some(Ordering::Greater) = self.data[cur].as_ref().unwrap().partial_cmp(self.data[next].as_ref().unwrap()) {
                    self.swap(cur, next);
                    cur = next;
                    continue;
                }
                break;
            }
        }
        result
    }
}


fn parent(idx: usize) -> usize {
    (idx - 1) / 2
}

fn lchild(idx: usize) -> usize {
    2 * idx + 1
}

fn rchild(idx: usize) -> usize {
    2 * idx + 2
}

