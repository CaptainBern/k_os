use x86::controlregs::cr2;

use crate::{idt::handler::Frame, interrupt_handler, paranoid_interrupt_handler, println};

interrupt_handler! {
    pub fn divide_by_zero(frame: Frame) {
        println!("Divide by zero: {:?}", frame);
    }
}

paranoid_interrupt_handler! {
    pub fn debug(frame: Frame) {
        println!("Debug: {:?}", frame);
    }
}

paranoid_interrupt_handler! {
    pub fn nmi(frame: Frame) {
        println!("NMI: {:?}", frame);
    }
}

interrupt_handler! {
    pub fn breakpoint(frame: Frame) {
        println!("Breakpoint: {:?}", frame);
    }
}

interrupt_handler! {
    pub fn overflow(frame: Frame) {
        println!("Overflow: {:?}", frame);
    }
}

interrupt_handler! {
    pub fn bound_range(frame: Frame) {
        println!("Bound-range: {:?}", frame);
    }
}

interrupt_handler! {
    pub fn invalid_opcode(frame: Frame) {
        println!("Invalid opcode: {:?}", frame);
    }
}

interrupt_handler! {
    pub fn device_not_available(frame: Frame) {
        println!("Device not available: {:?}", frame);
    }
}

interrupt_handler! {
    pub fn double_fault(frame: Frame, error: u64) {
        println!("Double fault: {:?}, error: {:#04x}", frame, error);
    }
}

interrupt_handler! {
    pub fn invalid_tss(frame: Frame, error: u64) {
        println!("Invalid TSS: {:?}, error: {:#04x}", frame, error);
    }
}

interrupt_handler! {
    pub fn segment_not_present(frame: Frame, error: u64) {
        println!("Segment not present: {:?}, {:#04x}", frame, error);
    }
}

interrupt_handler! {
    pub fn stack(frame: Frame, error: u64) {
        println!("Stack: {:?}, error: {:#04x}", frame, error);
    }
}

interrupt_handler! {
    pub fn general_protection(frame: Frame, error: u64) {
        println!("General protection: {:?}, error: {:#04x}", frame, error);
    }
}

interrupt_handler! {
    pub fn page_fault(frame: Frame, error: u64) {
        let addr = unsafe {
            cr2()
        };
        println!("Page fault: {:?}, error: {:#04b}, addr: {:#018x}", frame, error, addr);
    }
}

interrupt_handler! {
    pub fn x87_floating_point_ex(frame: Frame) {
        println!("x87 floating point exception pending: {:?}", frame);
    }
}

interrupt_handler! {
    pub fn alignment_check(frame: Frame, error: u64) {
        println!("Alignment check: {:?}, error: {:#04x}", frame, error);
    }
}

paranoid_interrupt_handler! {
    pub fn machine_check(frame: Frame) {
        println!("Machine check: {:?}", frame);
    }
}

interrupt_handler! {
    pub fn simd_floating_point(frame: Frame) {
        println!("SIMD floating point: {:?}", frame);
    }
}

interrupt_handler! {
    pub fn control_protection(frame: Frame, error: u64) {
        println!("Control protection: {:?}, error: {:#04x}", frame, error);
    }
}

interrupt_handler! {
    pub fn hypervisor_injection(frame: Frame) {
        println!("Hypervisor injection: {:?}", frame);
    }
}

paranoid_interrupt_handler! {
    pub fn vmm_communication(frame: Frame) {
        println!("VMM communication: {:?}", frame);
    }
}

paranoid_interrupt_handler! {
    pub fn security(frame: Frame) {
        println!("Security: {:?}", frame);
    }
}
