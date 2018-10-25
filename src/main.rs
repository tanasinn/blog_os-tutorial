#![no_std]
#![no_main]

extern crate bootloader_precompiled;
extern crate volatile;
#[macro_use]
extern crate lazy_static;
extern crate spin;

use core::panic::PanicInfo;

#[macro_use]
mod vga_buffer;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Num {} and {}", 13, 3.14);
    println!(".............................................");
    for i in 1..24 {
        println!("Row #{}", i);
    }
    panic!("PANIC!");

    loop {}
}
