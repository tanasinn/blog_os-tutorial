#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[cfg(test)]
extern crate std;
#[cfg(test)]
extern crate array_init;

extern crate bootloader_precompiled;
extern crate volatile;
#[macro_use]
extern crate lazy_static;
extern crate spin;

use core::panic::PanicInfo;

#[macro_use]
mod vga_buffer;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Num {} and {}", 13, 3.14);
    println!(".............................................");
    for i in 1..24 {
        println!("Row #{}", i);
    }
//    panic!("PANIC!");
    print!(".............................................");
    println!("\rAPA");
    print!("apa");
    print!("\x08elsin");

    loop {}
}
