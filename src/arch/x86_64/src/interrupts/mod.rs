//! This module deals with interrupts and exceptions. Exceptions are caused by
//! software execution errors, or internal processor errors. Like software
//! interrupts, exceptions are considered synchronous, as they are a result of
//! executing an instruction that causes the interrupt.
//!
//! Futher we have to differentiate between 3 types of exceptions:
//!  1. faults
//!  2. traps
//!  3. aborts
//! A fault will save the rIP that points to the faulting instruction. A trap
//! will save the rIP that points to the instruction *after* the faulting
//! instruction, which makes it a little easier to recover.
//! Aborts are generally unrecoverable and do not allow program restart.
//! To read more about interrupts and exceptions, refer to:
//!  - AMD Architecture Programmer's Manual Vol. 2, 8.1
//!  - Intel Software Developer Manual Vol. 3, 6.1

use core::mem;

use x86::{
    dtables::{lidt, DescriptorTablePointer},
    irq, segmentation, Ring,
};

use self::idt::{GateDescriptor, GateType};

pub mod handler;
pub mod idt;
pub mod traps;

/// The early descriptor table.
static mut EARLY_IDT: [GateDescriptor; 256] = [GateDescriptor::new(); 256];

/// Pointer to the early descriptor table.
static mut EARLY_IDT_PTR: DescriptorTablePointer<idt::GateDescriptor> = DescriptorTablePointer {
    base: 0 as *const usize as *const GateDescriptor,
    limit: 0,
};

/// Fixed vector-identification numbers. These are used for predefined
/// exception and interrupt conditions.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Vector {
    DivideByZeroError,
    Debug,
    NonMaskableInterrupt,
    Breakpoint,
    InvalidOpcode,
    DeviceNotAvailable,
    DoubleFault,
    // _Reserved0,
    InvalidTSS = 10,
    SegmentNotPresent,
    Stack,
    GeneralProtection,
    PageFault,
    // _Reserved1,
    X87FloatingpointEx = 16,
    AlignmentCheck,
    MachineCheck,
    SIMDFloatingPoint,
    // _Reserved2,
    ControlProtectionEx = 21,
    // _Reserved3,
    // _Reserved4,
    // _Reserved5,
    // _Reserved6,
    // _Reserved7,
    // _Reserved8,
    HypervisorInjectionEx = 28,
    VMMCommunicationEx,
    SecurityException,
}

unsafe fn set_early_handler(vector: Vector, handler: usize) {
    EARLY_IDT[vector as usize].set_offset(handler);
    EARLY_IDT[vector as usize].set_selector(segmentation::cs());
    EARLY_IDT[vector as usize]
        .options_mut()
        .set_type(GateType::Trap);
    EARLY_IDT[vector as usize].options_mut().set_p(true);
}

/// Initialise the early interrupt descriptor table.
pub unsafe fn init_early_idt() {
    unsafe fn set_entry(vector: Vector, f: unsafe extern "C" fn()) {
        let desc = &mut EARLY_IDT[vector as usize];
        desc.set_offset(f as usize);
        desc.set_selector(segmentation::cs());
        let mut options = desc.options_mut();
        options.set_p(true);
        options.set_type(GateType::Trap);
        options.set_dpl(Ring::Ring0);
    }

    set_entry(Vector::DivideByZeroError, traps::divide_by_zero);
    set_entry(Vector::Debug, traps::debug);
    set_entry(Vector::NonMaskableInterrupt, traps::nmi);
    set_entry(Vector::Breakpoint, traps::breakpoint);
    set_entry(Vector::InvalidOpcode, traps::invalid_opcode);
    set_entry(Vector::DeviceNotAvailable, traps::device_not_available);
    set_entry(Vector::DoubleFault, traps::double_fault);
    set_entry(Vector::InvalidTSS, traps::invalid_tss);
    set_entry(Vector::SegmentNotPresent, traps::segment_not_present);
    set_entry(Vector::Stack, traps::stack);
    set_entry(Vector::GeneralProtection, traps::general_protection);
    set_entry(Vector::PageFault, traps::page_fault);
    set_entry(Vector::X87FloatingpointEx, traps::x87_floating_point_ex);
    set_entry(Vector::AlignmentCheck, traps::alignment_check);
    set_entry(Vector::MachineCheck, traps::machine_check);
    set_entry(Vector::SIMDFloatingPoint, traps::simd_floating_point);
    set_entry(Vector::ControlProtectionEx, traps::control_protection);
    set_entry(Vector::HypervisorInjectionEx, traps::hypervisor_injection);
    set_entry(Vector::VMMCommunicationEx, traps::vmm_communication);
    set_entry(Vector::SecurityException, traps::security);

    EARLY_IDT_PTR.base = EARLY_IDT.as_ptr();
    EARLY_IDT_PTR.limit = ((mem::size_of::<GateDescriptor>() * 32) - 1) as u16;

    // enable interrupts.
    lidt(&EARLY_IDT_PTR);
    irq::enable();
}
