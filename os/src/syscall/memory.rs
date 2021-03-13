use crate::mm::{VirtAddr, MapPermission, is_mapped};
use crate::task::{current_user_token, mmap_current};
use crate::config::PAGE_SIZE;

pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    if start % PAGE_SIZE != 0 {
        return -1;
    }

    if (prot & !0x7) != 0 || (prot & 0x7) == 0 {
        return -1;
    }
    
    let mut cur = start;
    let end = start + len;
    let token = current_user_token();

    while cur < end {
        if is_mapped(token, cur, MapPermission::empty()) {
            return -1;
        }
        cur += PAGE_SIZE;
    }

    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(end);
    let mut permission = MapPermission::U;
    if (prot & 0x1) != 0 {
        permission |= MapPermission::R;
    }
    if (prot & 0x2) != 0 {
        permission |= MapPermission::W;
    }
    if (prot & 0x4) != 0 {
        permission |= MapPermission::X;
    }

    match mmap_current(start_va, end_va, permission) {
        Ok(_) => (cur - start) as isize,
        Err(_) => -1,
    }
}
