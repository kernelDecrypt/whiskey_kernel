/* TODO -> ADD MAP, UNMAP, TRANSLATE APIS, 
CREATE SEPARATE USER ADDRESS SPACE, PROPER SYSCALL POINTER VALIDATION */

use core::ptr;

const PAGE_SIZE: usize = 4096;
const SATP_MODE_SV39: usize = 8usize << 60;

const PTE_VALID: usize = 1 << 0;
const PTE_READ: usize = 1 << 1;
const PTE_WRITE: usize = 1 << 2;
const PTE_EXECUTE: usize = 1 << 3;
const PTE_ACCESSED: usize = 1 << 6;
const PTE_DIRTY: usize = 1 << 7;

#[repr(align(4096))]
struct RootPageTable([usize; 512]);

static mut ROOT_PAGE_TABLE: RootPageTable = RootPageTable([0; 512]);

pub fn init() {
    let root = unsafe { core::ptr::addr_of_mut!(ROOT_PAGE_TABLE.0) as *mut usize };

    unsafe {
        for index in 0..512 {
            ptr::write_volatile(root.add(index), 0);
        }

        map_gigabyte(root, 0, 0x0000_0000);
        map_gigabyte(root, 1, 0x4000_0000);
        map_gigabyte(root, 2, 0x8000_0000);

        let root_ppn = (root as usize) / PAGE_SIZE;
        let satp = SATP_MODE_SV39 | root_ppn;
        core::arch::asm!(
            "csrw satp, {satp}",
            "sfence.vma",
            satp = in(reg) satp,
            options(nostack, preserves_flags)
        );
    }
}

unsafe fn map_gigabyte(root: *mut usize, index: usize, physical_base: usize) {
    let ppn = physical_base >> 12;
    let permissions = PTE_VALID
        | PTE_READ
        | PTE_WRITE
        | PTE_EXECUTE
        | PTE_ACCESSED
        | PTE_DIRTY;
    ptr::write_volatile(root.add(index), (ppn << 10) | permissions);
}