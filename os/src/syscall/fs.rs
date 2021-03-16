use crate::mm::{UserBuffer, MapPermission, translated_byte_buffer, translated_refmut, is_mapped};
use crate::task::{current_user_token, current_task};
use crate::config::PAGE_SIZE;
use crate::fs::{make_pipe};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();

    let mut start = buf as usize / PAGE_SIZE * PAGE_SIZE;
    let end = start + len;
    while start < end {
        if !is_mapped(token, start, MapPermission::U | MapPermission::R) {
            return -1;
        }
        start += PAGE_SIZE;
    }

    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        drop(inner);
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        )
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();

    let mut start = buf as usize / PAGE_SIZE * PAGE_SIZE;
    let end = start + len;
    while start < end {
        if !is_mapped(token, start, MapPermission::U | MapPermission::W) {
            return -1;
        }
        start += PAGE_SIZE;
    }

    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        drop(inner);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        )
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();

    let mut start = pipe as usize / PAGE_SIZE * PAGE_SIZE;
    let end = start + 2 * core::mem::size_of::<usize>();
    while start < end {
        if !is_mapped(token, start, MapPermission::U | MapPermission::W) {
            return -1;
        }
        start += PAGE_SIZE;
    }

    let mut inner = task.acquire_inner_lock();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}
