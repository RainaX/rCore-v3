use crate::mm::{translated_byte_buffer, is_mapped};
use crate::task::current_user_token;
use crate::config::PAGE_SIZE;

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let mut start = buf as usize;
            let end = start + len;
            while start < end {
                if !is_mapped(current_user_token(), start) {
                    return -1;
                }
                start += PAGE_SIZE;
            }
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        },
        _ => {
            //panic!("Unsupported fd in sys_write!");
            -1
        }
    }
}
