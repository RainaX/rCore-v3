use super::File;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use lazy_static::*;
use crate::mm::UserBuffer;

const MAILBOX_CAPACITY: usize = 16;
pub const MAX_MAIL_LEN: usize = 256;


pub struct Mailbox {
    inner: Mutex<MailboxInner>,
}


lazy_static! {
    static ref MAILBOX_MANAGER: Mutex<BTreeMap<usize, Arc<Mailbox>>> = Mutex::new(BTreeMap::new());
}


impl Mailbox {
    pub fn new(pid: usize) -> Arc<Mailbox> {
        let mailbox = Arc::new(Mailbox {
            inner: Mutex::new(MailboxInner::new()),
        });
        MAILBOX_MANAGER.lock().insert(pid, mailbox.clone());
        mailbox
    }
}


pub struct MailboxInner {
    buffer: Vec<Vec<u8>>,
    head: usize,
    tail: usize,
    status: MailboxStatus,
}

impl MailboxInner {
    pub fn new() -> Self {
        Self {
            buffer: vec![vec![]; MAILBOX_CAPACITY],
            head: 0,
            tail: 0,
            status: MailboxStatus::EMPTY,
        }
    }

    pub fn read_mail(&mut self, buf: UserBuffer) -> usize {
        self.status = MailboxStatus::NORMAL;

        let mut read_size: usize = 0;
        let mut buf_iter = buf.into_iter();
        while let Some(byte_ref) = buf_iter.next() {
            if read_size >= self.buffer[self.head].len() {
                break;
            }
            unsafe { *byte_ref = self.buffer[self.head][read_size]; }
            read_size += 1;
        }
        self.buffer[self.head].clear();
        self.head = (self.head + 1) % MAILBOX_CAPACITY;
        if self.head == self.tail {
            self.status = MailboxStatus::EMPTY;
        }
        read_size
    }

    pub fn write_mail(&mut self, buf: UserBuffer) -> usize {
        self.status = MailboxStatus::NORMAL;

        let mut write_size: usize = 0;
        let mut buf_iter = buf.into_iter();
        while let Some(byte_ref) = buf_iter.next() {
            self.buffer[self.tail].push(unsafe { *byte_ref });
            write_size += 1;
        }
        self.tail = (self.tail + 1) % MAILBOX_CAPACITY;
        if self.tail == self.head {
            self.status = MailboxStatus::FULL;
        }
        write_size
    }
}

impl File for Mailbox {
    fn readable(&self) -> bool { 
        self.inner.lock().status != MailboxStatus::EMPTY
    }

    fn writable(&self) -> bool {
        self.inner.lock().status != MailboxStatus::FULL   
    }

    fn read(&self, buf: UserBuffer) -> usize {
        self.inner.lock().read_mail(buf)
    }

    fn write(&self, buf: UserBuffer) -> usize {
        self.inner.lock().write_mail(buf)
    }
}

pub fn find_mailbox(pid: usize) -> Option<Arc<Mailbox>> {
    MAILBOX_MANAGER.lock().get(&pid).map(|m| m.clone())
}

pub fn remove_mailbox(pid: usize) {
    MAILBOX_MANAGER.lock().remove(&pid);
}


#[derive(Copy, Clone, PartialEq)]
enum MailboxStatus {
    EMPTY,
    FULL,
    NORMAL,
}
