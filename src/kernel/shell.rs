/*The Serial Shell
"What the fuck"
*/

use crate::{drivers::plic, drivers::timer, drivers::uart, panic, tasks};
use {print, println};

pub fn run_shell() {
    let mut buffer = [0u8; 128];
    let mut len = 0usize;

    print_prompt();

    loop {
        // Heartbeat Testing. Won't be needed ever probably.

        /*if timer::heartbeat_pending() {
            println!("[heartbeat]");
            print_prompt();
        }  */

        if let Some(uart) = uart::get_uart() {
            if let Some(byte) = uart.read_byte() {
                match byte {
                    b'\r' | b'\n' => {
                        uart.write_byte(b'\r');

                        uart.write_byte(b'\n');
                        if let Some(cmd) = buffer.get(..len) {
                            handle_command(cmd);
                        }
                        len = 0;
                        print_prompt();
                    }
                    0x08 | 0x7f => {
                        if len > 0 {
                            len -= 1;
                            uart.write_byte(0x08);
                            uart.write_byte(b' ');
                            uart.write_byte(0x08);
                        }
                    }
                    _ => {
                        if len < buffer.len() {
                            buffer[len] = byte;
                            len += 1;
                            uart.write_byte(byte);
                        }
                    }
                }
            }
        }
    }
}

fn handle_command(cmd: &[u8]) {
    let line = match core::str::from_utf8(cmd) {
        Ok(s) => s.trim(),
        Err(_) => {
            println!("Invalid input.");
            return;
        }
    };

    if line.is_empty() {
        return;
    }

    let mut parts = line.split_ascii_whitespace();
    let command = parts.next().unwrap_or("");

    match command {
        "help" => {
            println!("Available commands:");
            println!("help -> show this message");
            println!("version -> show the version information");
            println!("echo -> print text back to the console");
            println!("clear -> clear the screen");
            println!("uptime -> show the approximate uptime in heartbeat intervals");
            println!("sleep <n> -> wait for n timer ticks");
            println!("heap_debug -> list debug info for memory");
            println!("tasks -> show registered tasks");
            println!("run_task -> run one scheduler round");
            println!("panic -> trigger a kernel panic");
        }
        "version" => {
            println!(
                "whiskey_kernel v{}",
                option_env!("VERSION").unwrap_or("unknown")
            );
        }
        "echo" => {
            let rest = &line[command.len()..].trim_start();
            println!("{}", rest);
        }
        "clear" => {
            print!("\x1b[2J\x1b[H");
        }
        "uptime" => {
            println!("uptime: {} heartbeat intervals", timer::uptime());
        }
        "sleep" => {
            let arg = parts.next().unwrap_or("0");
            match arg.parse::<u64>() {
                Ok(delay) => {
                    println!("sleeping for {} ticks", delay);
                    timer::wait_for_ticks(delay);
                    println!("done");
                }
                Err(_) => {
                    println!("usage: sleep <ticks>");
                }
            }
        }
        "heap_debug" => {
            crate::alloc::debug_info();
        }
        "tasks" => {
            tasks::dump_status();
        }
        "run_task" => {
            tasks::run_scheduler_once();
            println!("scheduler round complete");
        }
        "panic" => {
            println!("Triggering panic");
            panic::induce_panic();
        }
        _ => {
            println!("Unknown command: {}", command);
        }
    }
}

fn print_prompt() {
    if let Some(uart) = uart::get_uart() {
        uart.reset_color();
    }

    print!("# ");

    if let Some(uart) = uart::get_uart() {
        uart.set_color(uart::COLOR_BRIGHT_YELLOW);
    }
}
