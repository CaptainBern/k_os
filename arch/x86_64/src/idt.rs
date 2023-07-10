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

use spin::Once;
use x86::{
    dtables::{lidt, DescriptorTablePointer},
    segmentation::cs,
};

use crate::desc::{Access, GateDescriptor, GateDescriptorType};

pub mod handler;
pub mod traps;

/// The early descriptor table.
static mut EARLY_IDT: [GateDescriptor; 256] = [GateDescriptor::NULL; 256];

pub fn load() {
    unsafe {
        let ptr: DescriptorTablePointer<GateDescriptor> = DescriptorTablePointer {
            base: &EARLY_IDT as *const _,
            limit: ((32 * mem::size_of::<GateDescriptor>()) - 1) as u16,
        };

        lidt(&ptr);
    }
}

/// Set the IST for the given vector.
pub fn set_ist(vector: u8, ist: u8) {
    unsafe {
        EARLY_IDT[vector as usize].set_ist(ist);
    }
}

/// Initialise the early interrupt descriptor table.
pub fn init_once() {
    static INIT: Once<()> = Once::new();
    INIT.call_once(|| {
        unsafe fn set_gate(vector: u8, isr: unsafe extern "C" fn()) {
            EARLY_IDT[vector as usize] = GateDescriptor::new(
                isr as u64,
                cs(),
                GateDescriptorType::Trap,
                Access::DPL_0 | Access::P,
                0,
            );
        }

        unsafe {
            set_gate(0, traps::divide_by_zero);
            set_gate(1, traps::debug);
            set_gate(2, traps::nmi);
            set_gate(3, traps::breakpoint);
            set_gate(4, traps::overflow);
            set_gate(5, traps::bound_range);
            set_gate(6, traps::invalid_opcode);
            set_gate(7, traps::device_not_available);
            set_gate(8, traps::double_fault);
            set_gate(10, traps::invalid_tss);
            set_gate(11, traps::segment_not_present);
            set_gate(12, traps::stack);
            set_gate(13, traps::general_protection);
            set_gate(14, traps::page_fault);
            set_gate(16, traps::x87_floating_point_ex);
            set_gate(17, traps::alignment_check);
            set_gate(18, traps::machine_check);
            set_gate(19, traps::simd_floating_point);
            set_gate(21, traps::control_protection);
            set_gate(28, traps::hypervisor_injection);
            set_gate(29, traps::vmm_communication);
            set_gate(30, traps::security);
        }

        load();
    });
}
