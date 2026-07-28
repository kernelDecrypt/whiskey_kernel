use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

const CLINT_BASE: usize = 0x0200_0000;
const MTIMECMP: *mut u64 = (CLINT_BASE + 0x4000) as *mut u64;
const MTIME: *const u64 = (CLINT_BASE + 0x0bff8) as *const u64;
const TIMER_INTERVAL: u64 = 100_000;
const HEARTBEAT_INTERVAL: u64 = 100;

static TICK_COUNT: AtomicU64 = AtomicU64::new(0);
static HEARTBEAT_PENDING: AtomicBool = AtomicBool::new(false);

pub fn init_timer() {
    TICK_COUNT.store(0, Ordering::Relaxed);
    HEARTBEAT_PENDING.store(false, Ordering::Relaxed);
    set_next_timer();
}

fn set_next_timer() {
    unsafe {
        let now = read_volatile(MTIME);
        write_volatile(MTIMECMP, now.wrapping_add(TIMER_INTERVAL));
    }
}

#[no_mangle]
pub extern "C" fn handle_timer_interrupt() {
    set_next_timer();
    unsafe {
        core::arch::asm!("li t0, 0x2", "csrs mip, t0", out("t0") _);
    }
}

pub fn ticks() -> u64 {
    TICK_COUNT.load(Ordering::Relaxed)
}

pub fn uptime() -> u64 {
    ticks() / HEARTBEAT_INTERVAL
}

pub fn heartbeat_pending() -> bool {
    HEARTBEAT_PENDING.load(Ordering::Relaxed)
}

pub fn consume_heartbeat() -> bool {
    HEARTBEAT_PENDING.swap(false, Ordering::Relaxed)
}

pub fn wait_for_ticks(count: u64) {
    let target = ticks().wrapping_add(count);
    while ticks() < target {
        core::hint::spin_loop();
    }
}


#[no_mangle]
pub extern "C" fn handle_s_timer_tick() {
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);
    if TICK_COUNT.load(Ordering::Relaxed) % HEARTBEAT_INTERVAL == 0 {
        HEARTBEAT_PENDING.store(true, Ordering::Relaxed);
    }
}