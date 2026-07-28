/* PLIC: Platform level Interrupt Controller
The PLIC lets the kernel assign priority levels depending on urgency. */

// TODO: Comment complete ass rewrite

use core::ptr::{read_volatile, write_volatile};

const PLIC_BASE: usize = 0x0C00_0000;
const PLIC_PRIORITY: usize = PLIC_BASE + 0x0;
const PLIC_PENDING: usize = PLIC_BASE + 0x1000;
const PLIC_ENABLE: usize = PLIC_BASE + 0x2000;
const PLIC_CONTEXT_BASE: usize = PLIC_BASE + 0x200000;
const PLIC_ENABLE_STRIDE: usize = 0x80;
const PLIC_CONTEXT_STRIDE: usize = 0x1000;

pub fn init_plic_for_context(context: usize) {
    unsafe {
        let threshold = (PLIC_CONTEXT_BASE + context * PLIC_CONTEXT_STRIDE) as *mut u32;
        write_volatile(threshold, 0);
    }
}

pub fn set_priority(irq: usize, prio: u32) {
    unsafe {
        let p = (PLIC_PRIORITY + irq * 4) as *mut u32;
        write_volatile(p, prio);
    }
}

pub fn enable_irq_for_context(context: usize, irq: usize) {
    unsafe {
        let byte_off = (irq / 32) * 4;
        let bit = 1u32 << (irq % 32);
        let en = (PLIC_ENABLE + context * PLIC_ENABLE_STRIDE + byte_off) as *mut u32;
        let v = read_volatile(en);
        write_volatile(en, v | bit);
    }
}

pub fn claim(context: usize) -> u32 {
    unsafe {
        let claim = (PLIC_CONTEXT_BASE + context * PLIC_CONTEXT_STRIDE + 0x4) as *mut u32;
        read_volatile(claim)
    }
}

pub fn complete(context: usize, irq: u32) {
    unsafe {
        let claim = (PLIC_CONTEXT_BASE + context * PLIC_CONTEXT_STRIDE + 0x4) as *mut u32;
        write_volatile(claim, irq);
    }
}

// QEMU virt: context = hart*2 (M-mode), hart*2+1 (S-mode).
pub const fn m_context(hart: usize) -> usize { hart * 2 }
pub const fn s_context(hart: usize) -> usize { hart * 2 + 1 }