use crate::{drivers::uart, println, alloc};

pub const SYS_WRITE: usize = 1;
pub const SYS_EXIT: usize = 2;
pub const SYS_READ: usize = 3;



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

fn ok(value: usize) -> usize {
    value
}

fn err(e: SyscallError) -> usize {
    e.encode()
}

// Bounds check against kernel heap
fn validate_user_buffer(ptr: usize, len: usize) -> Result<(), SyscallError> {
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

            if fd != 1 {
                return err(SyscallError::BadFd);
            }
            if let Err(e) = validate_user_buffer(ptr, len) {
                return err(e);
            }

            let bytes = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
            if let Some(uart) = uart::get_uart() {
                uart.write_bytes(bytes);
            }
            ok(len)
        }
        SYS_EXIT => {
            println!("user exit request with status {}", arg0);
            loop {
                unsafe {
                    core::arch::asm!("wfi");
                }
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
            if let Err(e) = validate_user_buffer(ptr, 1) {
                return err(e);
            }

            if let Some(uart) = uart::get_uart() {
                if let Some(byte) = uart.read_byte() {
                    unsafe {
                        core::ptr::write(ptr as *mut u8, byte);
                    }
                    ok(1)
                } else {
                    ok(0)
                }
            } else {
                ok(0)
            }
        }
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
    }

    #[test]
    fn error_encoding_does_not_collide_with_plausible_success_values() {
        // nobody should write 2^64- 4 bytes through a 1mib heap.
        assert!(SyscallError::BadFd.encode() > 0xFFFF_FFFF_FFFF_0000);
    }
}