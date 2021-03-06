use super::{TimeVal, Stat};

const SYSCALL_DUP: usize = 24;
const SYSCALL_UNLINKAT: usize = 35;
const SYSCALL_LINKAT: usize = 37;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_FSTAT: usize = 80;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_SET_PRIORITY: usize = 140;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_MUNMAP: usize = 215;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_MMAP: usize = 222;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_SPAWN: usize = 400;
const SYSCALL_MAILREAD: usize = 401;
const SYSCALL_MAILWRITE: usize = 402;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (ret)
            : "{x10}" (args[0]), "{x11}" (args[1]), "{x12}" (args[2]), "{x17}" (id)
            : "memory"
            : "volatile"
        );
    }
    ret
}

fn syscall5(id: usize, args: [usize; 5]) -> isize {
    let mut ret: isize;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (ret)
            : "{x10}" (args[0]), "{x11}" (args[1]), "{x12}" (args[2]), "{x13}" (args[3]), "{x14}" (args[4]), "{x17}" (id)
            : "memory"
            : "volatile"
        );
    }
    ret
}


pub fn sys_dup(fd: usize) -> isize {
    syscall(SYSCALL_DUP, [fd, 0, 0])
}


pub fn sys_unlinkat(dirfd: usize, path: &str, flags: u32) -> isize {
    syscall(SYSCALL_UNLINKAT, [dirfd, path.as_ptr() as usize, flags as usize])
}


pub fn sys_linkat(
    olddirfd: usize,
    oldpath: &str,
    newdirfd: usize,
    newpath: &str,
    flags: u32,
    ) -> isize {
    syscall5(SYSCALL_LINKAT, [
        olddirfd,
        oldpath.as_ptr() as usize,
        newdirfd,
        newpath.as_ptr() as usize,
        flags as usize,
    ])
}


pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall(SYSCALL_OPEN, [path.as_ptr() as usize, flags as usize, 0])
}


pub fn sys_close(fd: usize) -> isize {
    syscall(SYSCALL_CLOSE, [fd, 0, 0])
}


pub fn sys_pipe(pipe: &mut [usize]) -> isize {
    syscall(SYSCALL_PIPE, [pipe.as_mut_ptr() as usize, 0, 0])
}


pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(SYSCALL_READ, [fd, buffer.as_ptr() as usize, buffer.len()])
}


pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}


pub fn sys_fstat(fd: usize, st: &mut Stat) -> isize {
    syscall(SYSCALL_FSTAT, [fd, st as *mut _ as usize, 0])
}


pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0]);
    panic!("sys_exit never returns!");
}


pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}


pub fn sys_set_priority(priority: isize) -> isize {
    syscall(SYSCALL_SET_PRIORITY, [priority as usize, 0, 0])
}


pub fn sys_get_time(time: &mut TimeVal, tz: usize) -> isize {
    syscall(SYSCALL_GET_TIME, [time as *mut _ as usize, tz, 0])
}


pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0])
}


pub fn sys_munmap(start: usize, len: usize) -> isize {
    syscall(SYSCALL_MUNMAP, [start, len, 0])
}


pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0])
}


pub fn sys_exec(path: &str, args: &[*const u8]) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, args.as_ptr() as usize, 0])
}


pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    syscall(SYSCALL_MMAP, [start, len, prot])
}


pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}

pub fn sys_spawn(path: &str) -> isize {
    syscall(SYSCALL_SPAWN, [path.as_ptr() as usize, 0, 0])
}

pub fn sys_mailread(buf: *const u8, len: usize) -> isize {
    syscall(SYSCALL_MAILREAD, [buf as usize, len, 0])
}

pub fn sys_mailwrite(pid: usize, buf: *const u8, len: usize) -> isize {
    syscall(SYSCALL_MAILWRITE, [pid, buf as usize, len])
}
