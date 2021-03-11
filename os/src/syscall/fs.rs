use crate::task::get_current_task_id;
use crate::loader::valid_app_buf;

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let app_id = get_current_task_id();
            if !valid_app_buf(app_id, buf as usize, len) {
                return -1;
            }
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        },
        _ => {
            //panic!("Unsupported fd in sys_write!");
            -1
        }
    }
}
