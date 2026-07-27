use crate::{alloc, drivers::{timer, uart}, println, tasks};

pub const SYS_WRITE: usize = 1;
pub const SYS_EXIT: usize = 2;
pub const SYS_READ: usize = 3;
pub const SYS_YIELD: usize = 4;
pub const SYS_GETPID: usize = 5;
pub const SYS_SLEEP: usize = 6;
pub const SYS_UPTIME: usize = 7;


/*
TODOS
SYS_YIELD is total bull until we are done with page tables
Rewrite this whole goddamn messy file */


/// syscall error codes encoded as ( usize::MAX - (code - 1) ) (the
/// same trick Linux uses) with negative errno cast to an unsigned return
/// register, legitimate byte counts will never get anywhere near usize::MAX,
/// so theres no error
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallError {
    BadFd = 1,
    BadPointer = 2,
    Overflow = 3,
    Unknown = 4,
}

impl SyscallError {
    fn encode(self) -> usize {
        usize::MAX - (self as usize - 1)
    }
}

fn ok(value: usize) -> usize { value }
fn err(e: SyscallError) -> usize { e.encode() 
}

/*
My own rough seam current just returns the same thing
once we get page tables current looks up every tasks page table */

trait AddressSpace {
    fn validate(&self, ptr: usize, len: usize) -> Result<(), SyscallError>;
}

struct FlatHeapSpace;

impl AddressSpace for FlatHeapSpace {
    fn validate(&self, ptr: usize, len: usize) -> Result<(), SyscallError> {
        if len == 0 {
            return Ok(());
        }
        if ptr == 0 {
            return Err(SyscallError::BadPointer);
        }
        let end = ptr.checked_add(len).ok_or(SyscallError::Overflow)?;
        let (heap_start, heap_end) = alloc::heap_bounds();
        if ptr < heap_start || end > heap_end {
            return Err(SyscallError::BadPointer);
        }
        Ok(())
    }
}

// will be swapped later
fn current_address_space() -> impl AddressSpace {
    FlatHeapSpace
}

#[no_mangle]
pub extern "C" fn rust_syscall_handler(
    syscall: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    _arg3: usize,
    mepc: usize,
) -> usize {
    match syscall {
        SYS_WRITE => {
            let fd = arg0;
            let ptr = arg1;
            let len = arg2;

            if fd != 1 && fd != 2 {
                return err(SyscallError::BadFd);
            }
            if let Err(e) = current_address_space().validate(ptr, len) {
                return err(e);
            }

            let bytes = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
            if let Some(uart) = uart::get_uart() {
                if fd == 2 {
                    uart.set_color(uart::COLOR_BRIGHT_RED);
                    uart.write_bytes(bytes);
                    uart.reset_color();
                } else {
                    uart.write_bytes(bytes);
                }
            }
            ok(len)
        }
        SYS_EXIT => {
            println!("user exit request with status {}", arg0);
            loop {
                unsafe { core::arch::asm!("wfi"); }
            }
        }
        SYS_READ => {
            let fd = arg0;
            let ptr = arg1;
            let len = arg2;

            if fd != 0 {
                return err(SyscallError::BadFd);
            }
            if len == 0 {
                return ok(0);
            }
            if let Err(e) = current_address_space().validate(ptr, 1) {
                return err(e);
            }

            if let Some(uart) = uart::get_uart() {
                if let Some(byte) = uart.read_byte() {
                    unsafe { core::ptr::write(ptr as *mut u8, byte); }
                    ok(1)
                } else {
                    ok(0)
                }
            } else {
                ok(0)
            }
        }
        SYS_YIELD => {
            // NOTE not a real context switch yet
            tasks::run_scheduler_once();
            ok(0)
        }
        SYS_GETPID => ok(tasks::current_task_id()),
        SYS_SLEEP => {
            timer::wait_for_ticks(arg0 as u64);
            ok(0)
        }
        SYS_UPTIME => ok(timer::uptime() as usize),
        _ => {
            println!("unknown syscall {} at {:#x}", syscall, mepc);
            err(SyscallError::Unknown)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syscall_numbers_are_defined() {
        assert_eq!(SYS_WRITE, 1);
        assert_eq!(SYS_EXIT, 2);
        assert_eq!(SYS_READ, 3);
        assert_eq!(SYS_YIELD, 4);
        assert_eq!(SYS_GETPID, 5);
        assert_eq!(SYS_SLEEP, 6);
        assert_eq!(SYS_UPTIME, 7);
    }
}