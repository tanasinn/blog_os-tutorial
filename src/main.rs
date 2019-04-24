#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]

//use blog_os::print;
use blog_os::println;
use bootloader::{bootinfo::BootInfo, entry_point};
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\x1B[41;97m{}\x1B[0m", info);
    blog_os::hlt_loop();
}

entry_point!(kernel_main);

#[cfg(not(test))]
#[allow(unconditional_recursion)]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
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

    use blog_os::memory;
    use x86_64::{structures::paging::Page, VirtAddr};

    let mut mapper = unsafe { memory::init(boot_info.physical_memory_offset) };
    let mut frame_allocator = memory::init_frame_allocator(&boot_info.memory_map);

    // map a previously unmapped page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    println!("It did not crash!");

    blog_os::hlt_loop();
}
