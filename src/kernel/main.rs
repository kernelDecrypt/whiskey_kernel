#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

use crate::tests::test_heap_alloc::run_heap_tests;


extern crate alloc as _alloc;
mod abi;
mod alloc;
mod drivers;
mod panic;
mod paging;
mod shell;
mod tasks;
mod trap;
mod tests;

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // Keep main simple..nice and short.

    alloc::alloc_init();
    paging::init();
    drivers::init_drivers();
    tasks::init();
    
    
    println!("Type 'help' for commands.");

    shell::run_shell();


    loop {}
}
