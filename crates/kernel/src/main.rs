//! Windows NT Unikernel - Bare Metal Kernel
//!
//! This is a minimal x86_64 kernel that will eventually run Windows PE binaries.
//! Based on phil-opp's "Writing an OS in Rust" tutorials.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

mod vga_buffer;
mod serial;

/// Kernel entry point
///
/// This function is called by the bootloader.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize VGA buffer
    vga_buffer::WRITER.lock().clear();

    // Initialize serial port for debugging
    serial::init();

    println!("Windows NT Unikernel v0.1.0");
    println!("Booting...");

    // TODO: Phase 2 implementation
    // 1. Initialize GDT
    // 2. Initialize IDT
    // 3. Initialize memory management
    // 4. Load PE binary from initrd
    // 5. Set up TEB/PEB
    // 6. Execute binary

    println!("Kernel initialization complete");
    println!("Halting...");

    hlt_loop();
}

/// Panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    hlt_loop();
}

/// Halt loop
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
