#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]

//use blog_os::print;
use blog_os::println;
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\x1B[41;97m{}\x1B[0m", info);
    blog_os::hlt_loop();
}

#[cfg(not(test))]
#[no_mangle]
#[allow(unconditional_recursion)]
pub extern "C" fn _start() -> ! {
    use blog_os::interrupts::PICS;

    blog_os::gdt::init();
    blog_os::interrupts::init_idt();

    unsafe {
        PICS.lock().initialize();
    };
    x86_64::instructions::interrupts::enable();


//    for row in 0..16 {
//        let bg_offset = if row < 8 { 40 } else { 100 - 8 };
//        for col in 0..16 {
//            let fg_offset = if col < 8 { 30 } else { 90 - 8 };
//            print!("\x1B[{};{}m{:X}", bg_offset + row, fg_offset + col, col);
//        }
//        println!("\x1B[0m")
//    }
//    println!();
//    print!("> ");

//    x86_64::instructions::int3();
//
//    fn stack_overflow() {
//        stack_overflow(); // for each recursion, the return address is pushed
//    }
//
//    // trigger a stack overflow
//    stack_overflow();

//    let ptr = 0xcafebabe as *mut u32;
//    unsafe {
//        *ptr = 42;
//    }

    use x86_64::registers::control::Cr3;

    let (l4pt, _) = Cr3::read();
    println!("L4 pt @: {:?}", l4pt.start_address());
    println!();

    use x86_64::structures::paging::PageTable;

    let level_4_table_ptr = 0xffff_ffff_ffff_f000 as *const PageTable;
    let level_4_table = unsafe { &*level_4_table_ptr };
    for i in 0..10 {
        println!("Entry {}: {:?}", i, level_4_table[i]);
    }

    println!("It did not crash!");

    blog_os::hlt_loop();
}
