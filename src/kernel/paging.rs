/* TODO -> attach address spaces to tasks and switch them during schedulin */

use core::ptr;
use _alloc::alloc::{alloc, Layout};

const PAGE_SIZE: usize = 4096;
const PAGE_MASK: usize = PAGE_SIZE - 1;
const PAGE_TABLE_ENTRIES: usize = 512;
const SATP_MODE_SV39: usize = 8usize << 60;

pub const PTE_VALID: usize = 1 << 0;
pub const PTE_READ: usize = 1 << 1;
pub const PTE_WRITE: usize = 1 << 2;
pub const PTE_EXECUTE: usize = 1 << 3;
pub const PTE_USER: usize = 1 << 4;
const PTE_ACCESSED: usize = 1 << 6;
const PTE_DIRTY: usize = 1 << 7;

#[repr(align(4096))]
struct RootPageTable([usize; 512]);

static mut ROOT_PAGE_TABLE: RootPageTable = RootPageTable([0; 512]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapError {
    Unaligned,
    AlreadyMapped,
    InvalidFlags,
    OutOfMemory,
    UnsupportedPageSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnmapError {
    Unaligned,
    NotMapped,
    UnsupportedPageSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRangeError {
    Overflow,
    Unmapped,
    NotUser,
    Permission,
}

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

pub fn root_page_table() -> usize {
    unsafe { core::ptr::addr_of!(ROOT_PAGE_TABLE.0) as *const usize as usize }
}

pub unsafe fn create_address_space() -> Option<usize> {
    let table = allocate_table()?;
    let kernel_root = root_page_table() as *const usize;

    for index in 0..PAGE_TABLE_ENTRIES {
        let entry = ptr::read_volatile(kernel_root.add(index));
        ptr::write_volatile(table.add(index), entry);
    }

    Some(table as usize)
}

pub unsafe fn activate(root: usize) {
    let satp = SATP_MODE_SV39 | (root / PAGE_SIZE);
    core::arch::asm!(
        "csrw satp, {satp}",
        "sfence.vma",
        satp = in(reg) satp,
        options(nostack, preserves_flags)
    );
}

fn vpn(virtual_address: usize, level: usize) -> usize {
    (virtual_address >> (12 + level * 9)) & 0x1ff
}

fn make_pte(physical_address: usize, flags: usize) -> usize {
    (physical_address >> 12 << 10) | flags
}

fn pte_physical_address(pte: usize) -> usize {
    (pte >> 10) << 12
}

fn is_leaf(pte: usize) -> bool {
    pte & (PTE_READ | PTE_WRITE | PTE_EXECUTE) != 0
}

unsafe fn allocate_table() -> Option<*mut usize> {
    let layout = Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).ok()?;
    let table = alloc(layout) as *mut usize;
    if table.is_null() {
        return None;
    }

    ptr::write_bytes(table, 0, PAGE_TABLE_ENTRIES);
    Some(table)
}

unsafe fn walk(root: *mut usize, virtual_address: usize, create: bool)
    -> Option<(*mut usize, usize)>
{
    let mut table = root;

    for level in (0..=2).rev() {
        let entry = table.add(vpn(virtual_address, level));
        let pte = ptr::read_volatile(entry);

        if pte & PTE_VALID == 0 {
            if !create || level == 0 {
                return Some((entry, level));
            }

            let child = allocate_table()?;
            ptr::write_volatile(entry, make_pte(child as usize, PTE_VALID));
            table = child;
            continue;
        }

        if is_leaf(pte) {
            return Some((entry, level));
        }

        table = pte_physical_address(pte) as *mut usize;
    }

    None
}

pub unsafe fn map_page(
    root: usize,
    virtual_address: usize,
    physical_address: usize,
    flags: usize,
) -> Result<(), MapError> {
    if virtual_address & PAGE_MASK != 0 || physical_address & PAGE_MASK != 0 {
        return Err(MapError::Unaligned);
    }

    let leaf_flags = PTE_READ | PTE_WRITE | PTE_EXECUTE;
    if flags & PTE_VALID == 0 || flags & leaf_flags == 0 {
        return Err(MapError::InvalidFlags);
    }

    let (entry, level) = walk(root as *mut usize, virtual_address, true)
        .ok_or(MapError::OutOfMemory)?;
    if level != 0 {
        return Err(MapError::UnsupportedPageSize);
    }
    if ptr::read_volatile(entry) & PTE_VALID != 0 {
        return Err(MapError::AlreadyMapped);
    }

    ptr::write_volatile(entry, make_pte(physical_address, flags));
    flush_page(virtual_address);
    Ok(())
}

pub unsafe fn unmap_page(
    root: usize,
    virtual_address: usize,
) -> Result<usize, UnmapError> {
    if virtual_address & PAGE_MASK != 0 {
        return Err(UnmapError::Unaligned);
    }

    let (entry, level) = walk(root as *mut usize, virtual_address, false)
        .ok_or(UnmapError::NotMapped)?;
    if level != 0 {
        return Err(UnmapError::UnsupportedPageSize);
    }

    let pte = ptr::read_volatile(entry);
    if pte & PTE_VALID == 0 {
        return Err(UnmapError::NotMapped);
    }

    ptr::write_volatile(entry, 0);
    flush_page(virtual_address);
    Ok(pte_physical_address(pte))
}

pub unsafe fn translate(root: usize, virtual_address: usize) -> Option<usize> {
    let (entry, level) = walk(root as *mut usize, virtual_address, false)?;
    let pte = ptr::read_volatile(entry);
    if pte & PTE_VALID == 0 || is_leaf(pte) == false {
        return None;
    }

    let page_bits = 12 + level * 9;
    let offset_mask = (1usize << page_bits) - 1;
    Some(pte_physical_address(pte) | (virtual_address & offset_mask))
}

pub fn validate_user_range(
    pointer: usize,
    length: usize,
    required_flags: usize,
) -> Result<(), UserRangeError> {
    if length == 0 {
        return Ok(());
    }

    let end = pointer
        .checked_add(length)
        .ok_or(UserRangeError::Overflow)?;
    let page_alignment_mask = usize::MAX ^ PAGE_MASK;
    let first_page = pointer & page_alignment_mask;
    let last_page = (end - 1) & page_alignment_mask;
    let root = root_page_table();
    let mut page = first_page;

    loop {
        let pte = unsafe {
            let (entry, level) = walk(root as *mut usize, page, false)
                .ok_or(UserRangeError::Unmapped)?;
            if level != 0 {
                return Err(UserRangeError::Permission);
            }
            ptr::read_volatile(entry)
        };

        if pte & PTE_VALID == 0 {
            return Err(UserRangeError::Unmapped);
        }
        if pte & PTE_USER == 0 {
            return Err(UserRangeError::NotUser);
        }
        if pte & required_flags != required_flags {
            return Err(UserRangeError::Permission);
        }

        if page == last_page {
            break;
        }
        page = page
            .checked_add(PAGE_SIZE)
            .ok_or(UserRangeError::Overflow)?;
    }

    Ok(())
}

unsafe fn flush_page(virtual_address: usize) {
    core::arch::asm!(
        "sfence.vma {address}, zero",
        address = in(reg) virtual_address,
        options(nostack, preserves_flags)
    );
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