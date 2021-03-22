use core::cmp::Ordering;
use alloc::vec::Vec;

pub struct BinaryHeap<T: PartialOrd + Copy> {
    data: Vec<Option<T>>,
}


impl<T: PartialOrd + Copy> BinaryHeap<T> {
    fn swap(&mut self, i: usize, j: usize) {
        let temp = self.data[i].take();
        self.data[i] = self.data[j].take();
        self.data[j] = temp;
    }

    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    pub fn insert(&mut self, ele: T) {
        // insert the new element to the tail of array
        let mut cur = self.data.len();
        self.data.push(Some(ele));

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
        if self.data.len() == 0 {
            return None;
        }

        // Remove the element at head
        // Then move the tail element to head position
        let result = self.data[0].take();
        let tail_index = self.data.len() - 1;
        let tail = self.data[tail_index].take();
        self.data[0] = tail;
        self.data.pop().unwrap();

        // Swap until current element is not greater than both children
        if self.data.len() > 0 {
            let mut cur = 0;
            loop {
                let mut next = cur;

                let l = lchild(cur);
                let r = rchild(cur);

                let size = self.data.len();

                if l < size && r < size {
                    next = match self.data[l].as_ref().unwrap().partial_cmp(self.data[r].as_ref().unwrap()) {
                        Some(Ordering::Less) => l,
                        Some(Ordering::Greater) => r,
                        _ => l,
                    };
                } else if l < size {
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
