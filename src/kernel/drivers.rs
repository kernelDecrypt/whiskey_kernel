pub mod plic;
pub mod timer;
pub mod uart;

use crate::trap;

pub fn init_drivers() {
    uart::init_uart();

    let ctx = plic::s_context(0);
    plic::init_plic_for_context(ctx);

    plic::set_priority(10, 1);
    plic::enable_irq_for_context(ctx, 10);

    timer::init_timer();
    trap::enable_interrupts();
}