use riscv::register::time;
use crate::sbi::set_timer;
use crate::config::CLOCK_FREQ;

const TICKS_PER_SEC: usize = 100;
//const MSEC_PER_SEC: usize = 1000;
const USEC_PER_SEC: usize = 1000000;

pub fn get_time() -> usize {
    time::read()
}

/*
pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
}*/


pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}

pub fn get_time_val() -> TimeVal {
    let mut clock = time::read();
    let sec = clock / CLOCK_FREQ;
    clock %= CLOCK_FREQ;
    let usec = clock / (CLOCK_FREQ / USEC_PER_SEC);
    TimeVal(sec, usec)
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TimeVal(usize, usize);

