#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

extern crate alloc;

use user_lib::{fork, close, pipe, read, write, wait, get_time};
use alloc::format;

const LENGTH: usize = 3000;
#[no_mangle]
pub fn main() -> i32 {
    let mut down_pipe_fd = [0usize; 2];
    let mut up_pipe_fd = [0usize; 2];
    pipe(&mut down_pipe_fd);
    pipe(&mut up_pipe_fd);
    let mut random_str = [0u8; LENGTH];
    if fork() == 0 {
        close(down_pipe_fd[1]);
        close(up_pipe_fd[0]);
        assert_eq!(read(down_pipe_fd[0], &mut random_str) as usize, LENGTH);
        close(down_pipe_fd[0]);
        let sum: usize = random_str.iter().map(|v| *v as usize).sum::<usize>();
        println!("sum = {}(child)", sum);
        let sum_str = format!("{}", sum);
        write(up_pipe_fd[1], sum_str.as_bytes());
        close(up_pipe_fd[1]);
        println!("Child process exited!");
        0
    } else {
        close(down_pipe_fd[0]);
        close(up_pipe_fd[1]);
        for i in 0..LENGTH {
            random_str[i] = get_time() as u8;
        }

        assert_eq!(write(down_pipe_fd[1], &random_str) as usize, random_str.len());
        close(down_pipe_fd[1]);
        let sum: usize = random_str.iter().map(|v| *v as usize).sum::<usize>();
        println!("sum = {}(parent)", sum);
        let mut child_result = [0u8; 32];
        let result_len = read(up_pipe_fd[0], &mut child_result) as usize;
        close(up_pipe_fd[0]);
        assert_eq!(
            sum,
            str::parse::<usize>(
                core::str::from_utf8(&child_result[..result_len]).unwrap()
            ).unwrap()
        );
        let mut _unused: i32 = 0;
        wait(&mut _unused);
        println!("pipe_large_test passed!");
        0
    }
}
