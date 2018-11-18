#![feature(abi_x86_interrupt)]
#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate blog_os;
extern crate x86_64;
extern crate lazy_static;

use blog_os::exit_qemu;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use x86_64::structures::idt::{ExceptionStackFrame, InterruptDescriptorTable};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(blog_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn double_fault_handler(_stack_frame: &mut ExceptionStackFrame, _error_code: u64) {
    serial_println!("ok");

    unsafe {
        exit_qemu();
    }
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("failed");
    serial_println!("{}", info);
    unsafe {
        exit_qemu();
    }

    loop {}
}

#[cfg(not(test))]
#[no_mangle]
#[allow(unconditional_recursion)]
pub extern "C" fn _start() -> ! {
    blog_os::gdt::init();
    init_test_idt();

    fn stack_overflow() {
        stack_overflow();
    }

    // trigger a stack overflow
    stack_overflow();

    serial_println!("failed");
    serial_println!("No double fault exception occured");

    unsafe {
        exit_qemu();
    }

    loop {}
}