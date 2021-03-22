mod inode;
mod mailbox;
mod pipe;
mod stdio;

use crate::mm::UserBuffer;
use easy_fs::Stat;
pub trait File : Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
    fn fstat(&self) -> Option<Stat>;
}

pub use inode::{OSInode, open_file, link_file, unlink_file, OpenFlags, list_apps};
pub use mailbox::{Mailbox, MAX_MAIL_LEN, find_mailbox, remove_mailbox};
pub use pipe::{Pipe, make_pipe};
pub use stdio::{Stdin, Stdout};
