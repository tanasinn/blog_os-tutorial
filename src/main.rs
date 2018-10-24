#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Linux:
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}

// Windows:
#[no_mangle]
pub extern "C" fn mainCRTStartup() -> ! {
    main();
}

// MacOS / Windows:
#[no_mangle]
pub extern "C" fn main() -> ! {
    loop {}
}
