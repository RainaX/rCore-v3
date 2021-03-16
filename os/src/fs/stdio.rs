use super::File;
use crate::mm::{UserBuffer};
use crate::sbi::console_getchar;
use crate::task::suspend_current_and_run_next;

pub struct Stdin;

pub struct Stdout;

impl File for Stdin {
    fn read(&self, mut user_buf: UserBuffer) -> isize {
        assert_eq!(user_buf.len(), 1);
        let mut c: usize;
        loop {
            c = console_getchar();
            if c == 0 {
                suspend_current_and_run_next();
                continue;
            } else {
                break;
            }
        }
        let ch = c as u8;
        unsafe { user_buf.buffers[0].as_mut_ptr().write_volatile(ch); }
        1
    }

    fn write(&self, _user_buf: UserBuffer) -> isize {
        //panic!("Cannot write to stdin!");
        -1
    }
}

impl File for Stdout {
    fn read(&self, _user_buf: UserBuffer) -> isize {
        //panic!("Cannot read from stdout!");
        -1
    }

    fn write(&self, user_buf: UserBuffer) -> isize {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        user_buf.len() as isize
    }
}
