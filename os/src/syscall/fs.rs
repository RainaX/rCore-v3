use crate::mm::{
    UserBuffer,
    MapPermission,
    translated_byte_buffer,
    translated_str,
    translated_refmut,
    is_mapped};
use crate::task::{current_user_token, current_task};
use crate::config::PAGE_SIZE;
use crate::fs::{
    File,
    MAX_MAIL_LEN,
    find_mailbox,
    make_pipe,
    OpenFlags,
    open_file,
};
use alloc::sync::Arc;

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
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        drop(inner);
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
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
        if !file.readable() {
            return -1;
        }
        if fd == 0 && len != 1 {
            return -1;
        }
        let file = file.clone();
        drop(inner);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = match translated_str(token, path) {
        Some(path) => path,
        None => return -1,
    };
    if let Some(inode) = open_file(
        path.as_str(),
        OpenFlags::from_bits(flags).unwrap(),
    ) {
        let mut inner = task.acquire_inner_lock();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
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

pub fn sys_dup(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let new_fd = inner.alloc_fd();
    inner.fd_table[new_fd] = Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
    new_fd as isize
}

pub fn sys_mailread(buf: *mut u8, len: usize) -> isize {
    let token = current_user_token();
    let len = len.min(MAX_MAIL_LEN);

    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    let mailbox = inner.mailbox.clone();
    drop(inner);

    if !mailbox.readable() {
        return -1;
    }

    if len == 0 {
        0
    } else {
        let mut start = buf as usize / PAGE_SIZE * PAGE_SIZE;
        let end = start + len;
        while start < end {
            if !is_mapped(token, start, MapPermission::U | MapPermission::W) {
                return -1;
            }
            start += PAGE_SIZE;
        }
        mailbox.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    }
}

pub fn sys_mailwrite(pid: usize, buf: *mut u8, len: usize) -> isize {
    let token = current_user_token();
    let len = len.min(MAX_MAIL_LEN);

    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    let mailbox;
    if task.pid.0 == pid {
        mailbox = inner.mailbox.clone();
    } else {
        mailbox = match find_mailbox(pid) {
            Some(mailbox) => mailbox,
            None => return -1,
        };
    }
    drop(inner);

    if !mailbox.writable() {
        return -1;
    }

    if len == 0 {
        0
    } else {
        let mut start = buf as usize / PAGE_SIZE * PAGE_SIZE;
        let end = start + len;
        while start < end {
            if !is_mapped(token, start, MapPermission::U | MapPermission::R) {
                return -1;
            }
            start += PAGE_SIZE;
        }
        mailbox.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    }
}
