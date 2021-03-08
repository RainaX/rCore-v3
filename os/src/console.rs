use crate::sbi::console_putchar;
use core::fmt::{self, Write};

struct Stdout;


impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}


pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}


pub fn print_in_color(args: fmt::Arguments, color_code: u8) {
    print(format_args!("\x1b[{}m{}\x1b[0m", color_code, args));
}


#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}


#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}


