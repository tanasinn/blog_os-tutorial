// LLVM throws an error if a function with the x86-interrupt calling convention is compiled
// for a Windows system.
#![cfg(not(windows))]

extern crate x86_64;

use x86_64::structures::idt::{InterruptDescriptorTable, ExceptionStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}