use core::arch::asm;
use core::fmt::{self, Write};
use core::sync::atomic::Ordering;

const UART_BASE: usize = 0x1000_0000;

const THR: usize = 0; // Transmit Holding Register (write here to send)
const RBR: usize = 0; // Receive Buffer Register (read here to receive)
const IER: usize = 1; // Interrupt Enable Register
const LSR: usize = 5; // Line Status Register

const LSR_THRE: u8 = 1 << 5; // Transmit Holding Register Empty bit
const LSR_DR: u8 = 1 << 0; // Data Ready bit
const RX_BUFFER_SIZE: usize = 256;

pub struct Uart {
    base: usize,
}

impl Uart {
    pub const fn new(base: usize) -> Self {
        Uart { base }
    }

    fn reg(&self, offset: usize) -> *mut u8 {
        (self.base + offset) as *mut u8
    }

    fn disable_interrupts() -> usize {
        let mut sstatus: usize;
        unsafe {
            asm!("csrrc {0}, sstatus, 0x8", out(reg) sstatus);
        }
        sstatus
    }

    fn restore_interrupts(mask: usize) {
        unsafe {
            if mask & 0x8 != 0 {
                asm!("csrs sstatus, 0x8");
            } else {
                asm!("csrc sstatus, 0x8");
            }
        }
    }

    fn push_rx_byte(&self, byte: u8) {
        let prev = Self::disable_interrupts();
        unsafe {
            if RX_LEN < RX_BUFFER_SIZE {
                RX_BUFFER[RX_TAIL] = byte;
                RX_TAIL = (RX_TAIL + 1) % RX_BUFFER_SIZE;
                RX_LEN += 1;
            }
        }
        Self::restore_interrupts(prev);
    }

    fn pop_rx_byte(&self) -> Option<u8> {
        let prev = Self::disable_interrupts();
        let byte = unsafe {
            if RX_LEN == 0 {
                None
            } else {
                let byte = RX_BUFFER[RX_HEAD];
                RX_HEAD = (RX_HEAD + 1) % RX_BUFFER_SIZE;
                RX_LEN -= 1;
                Some(byte)
            }
        };
        Self::restore_interrupts(prev);
        byte
    }

    fn drain_rx_fifo(&self) {
        loop {
            unsafe {
                if core::ptr::read_volatile(self.reg(LSR)) & LSR_DR == 0 {
                    break;
                }
                let byte = core::ptr::read_volatile(self.reg(RBR));
                self.push_rx_byte(byte);
            }
        }
    }

    pub fn write_byte(&self, byte: u8) {
        unsafe {
            while core::ptr::read_volatile(self.reg(LSR)) & LSR_THRE == 0 {}
            core::ptr::write_volatile(self.reg(THR), byte);
        }
    }

    pub fn write_bytes(&self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_byte(byte);
        }
    }

    pub fn read_byte(&self) -> Option<u8> {
        self.pop_rx_byte()
    }

    pub fn handle_interrupt(&self) {
        self.drain_rx_fifo();
    }

    fn write_ansi_code(&self, code: u8) {
        self.write_byte(0x1b); // ESCAPE
        self.write_byte(b'[');

        if code >= 100 {
            self.write_byte(b'1');
            self.write_byte(b'0' + (code - 100) as u8);
        } else {
            self.write_byte(b'0' + (code / 10) as u8);
            self.write_byte(b'0' + (code % 10) as u8);
        }

        self.write_byte(b'm');
    }

    pub fn set_color(&self, color: u8) {
        self.write_ansi_code(color);
    }

    pub fn reset_color(&self) {
        self.write_ansi_code(0);
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

// Global UART instance
static mut UART: Uart = Uart { base: UART_BASE };
static UART_INITIALIZED: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);
static mut RX_BUFFER: [u8; RX_BUFFER_SIZE] = [0; RX_BUFFER_SIZE];
static mut RX_HEAD: usize = 0;
static mut RX_TAIL: usize = 0;
static mut RX_LEN: usize = 0;

pub fn init_uart() {
    unsafe {
        core::ptr::write_volatile((UART_BASE + IER) as *mut u8, 0x01);
    }
    UART_INITIALIZED.store(true, Ordering::Release);
}

pub fn handle_uart_interrupt() {
    if let Some(uart) = get_uart() {
        uart.handle_interrupt();
    }
}

pub fn get_uart() -> Option<&'static mut Uart> {
    if UART_INITIALIZED.load(Ordering::Acquire) {
        unsafe { Some(&mut UART) }
    } else {
        None
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        if let Some(uart) = $crate::drivers::uart::get_uart() {
            use core::fmt::Write;
            let _ = write!(uart, $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
    };
}

// Color constants (ANSI codes)
pub const COLOR_BLACK: u8 = 30;
pub const COLOR_RED: u8 = 31;
pub const COLOR_GREEN: u8 = 32;
pub const COLOR_YELLOW: u8 = 33;
pub const COLOR_BLUE: u8 = 34;
pub const COLOR_MAGENTA: u8 = 35;
pub const COLOR_CYAN: u8 = 36;
pub const COLOR_WHITE: u8 = 37;

pub const COLOR_BRIGHT_RED: u8 = 91;
pub const COLOR_BRIGHT_GREEN: u8 = 92;
pub const COLOR_BRIGHT_YELLOW: u8 = 93;
pub const COLOR_BRIGHT_BLUE: u8 = 94;
pub const COLOR_BRIGHT_MAGENTA: u8 = 95;
pub const COLOR_BRIGHT_CYAN: u8 = 96;
pub const COLOR_BRIGHT_WHITE: u8 = 97;
