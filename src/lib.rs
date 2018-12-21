#![no_std]
#![feature(abi_x86_interrupt)]

extern crate bootloader_precompiled;
#[macro_use]
extern crate lazy_static;
extern crate pc_keyboard;
extern crate pic8259_simple;
extern crate spin;
extern crate uart_16550;
extern crate volatile;
extern crate x86_64;

#[cfg(test)]
extern crate std;
#[cfg(test)]
extern crate array_init;

#[macro_use]
pub mod vga_buffer;
pub mod gdt;
pub mod interrupts;
pub mod serial;

pub unsafe fn exit_qemu() {
    use x86_64::instructions::port::Port;

    let mut port = Port::<u32>::new(0xF4);
    port.write(0);
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}