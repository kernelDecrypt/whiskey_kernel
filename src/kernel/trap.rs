

use drivers::{plic, uart};
use {print, println};

static mut EXTERNAL_IRQ_PENDING: bool = false;

pub fn init() {
    unsafe {
        EXTERNAL_IRQ_PENDING = false;
    }
}

pub fn external_irq_pending() -> bool {
    unsafe { EXTERNAL_IRQ_PENDING }
}

pub fn clear_external_irq_pending() {
    unsafe {
        EXTERNAL_IRQ_PENDING = false;
    }
}

fn trap_desc(cause: usize) -> (&'static str, usize) {
    let interrupt = (cause >> 63) & 1 != 0;
    let code = cause & 0x7ff;

    if interrupt {
        let kind = match code {
            7 => "machine timer interrupt",
            11 => "machine external interrupt",
            _ => "unknown interrupt",
        };
        (kind, code)
    } else {
        let kind = match code {
            2 => "illegal instruction",
            3 => "breakpoint",
            5 => "load access fault",
            7 => "store/AMO access fault",
            _ => "unknown exception",
        };
        (kind, code)
    }
}

pub fn enable_interrupts() {
    unsafe {
        core::arch::asm!(
            "li t0, 0x2",      // SIE bit is in sstatus
            "csrs sstatus, t0",
            "li t0, 0x222",    // SSIE(0x2)STIE(0x20) SEIE(0x200)
            "csrs sie, t0",
        );
    }
}

#[no_mangle]
pub extern "C" fn handle_external_interrupt() {
    let ctx = plic::s_context(0);
    let irq = plic::claim(ctx);

    if irq != 0 {
        unsafe {
            EXTERNAL_IRQ_PENDING = true;
        }

        match irq {
            10 => {
                uart::handle_uart_interrupt();
            }
            _ => {}
        }

        plic::complete(ctx, irq);
    }
}

#[no_mangle]
pub extern "C" fn rust_exception_handler(
    mcause: usize,
    mepc: usize,
    mtval: usize,
    regs: *const usize,
) {
    let (kind, code) = trap_desc(mcause);

    println!("Exception caught:");
    println!("  mcause -> {:#x}", mcause);
    println!("  kind  -> {}", kind);
    println!("  code  -> {:#x}", code);
    println!("  mepc  -> {:#x}", mepc);
    println!("  mtval -> {:#x}", mtval);

    if !regs.is_null() {
        unsafe {
            // The assembly saves 30 registers starting at offset 16(sp).
            let regs_slice = core::slice::from_raw_parts(regs, 30);
            println!("Saved registers (low->high):");
            for (i, r) in regs_slice.iter().enumerate() {
                println!("  [{}] = {:#018x}", i, r);
            }
        }
    }

    println!("halting for debug (wfi loop)");
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
