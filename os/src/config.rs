pub const USER_STACK_SIZE: usize = 4096;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const MAX_APP_NUM: usize = 16;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;

pub const CLOCK_FREQ: usize = 12500000;

// Maximum running time for each app in seconds
// Should be very large, but set to 20 seconds to pass tests
// in reasonable time.
pub const MAX_APP_SEC: usize = 20;
