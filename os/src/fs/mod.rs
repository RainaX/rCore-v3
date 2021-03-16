mod pipe;
mod stdio;

use crate::mm::UserBuffer;
pub trait File : Send + Sync {
    fn read(&self, buf: UserBuffer) -> isize;
    fn write(&self, buf: UserBuffer) -> isize;
}

pub use pipe::{Pipe, make_pipe};
pub use stdio::{Stdin, Stdout};
