use core::mem;

use x86::{dtables::DescriptorTablePointer, irq, segmentation::cs, Ring};

use crate::{interrupts::traps, println};

/// The early interrupt descriptor table. This table only serves to get us
/// through the early setup code. Afterwards, the boot cpu will switch to its
/// own IDT, followed by booting all the other processors, where each of those
/// will initially use this table as well, after which they too switch to their
/// own tables.
static mut EARLY_IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

static mut EARLY_IDT_PTR: DescriptorTablePointer<Entry> = DescriptorTablePointer {
    limit: 0,
    base: 0 as *const Entry,
};

#[derive(Debug)]
#[repr(packed)]
pub struct InterruptDescriptorTable {
    entries: [Entry; 256],
}

impl InterruptDescriptorTable {
    pub const fn new() -> Self {
        InterruptDescriptorTable {
            entries: [Entry::new(); 256],
        }
    }

    /// Return a pointer to the underlying table.
    pub fn as_ptr(&self) -> *const Entry {
        self.entries.as_ptr()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum GateType {
    Interrupt = 0xe,
    Trap = 0xf,
}

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct Entry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    flags: u8,
    offset_middle: u16,
    offset_high: u32,
    _reserved: u32,
}

impl Entry {
    pub const fn new() -> Self {
        Entry {
            offset_low: 0,
            selector: 0,
            ist: 0,
            flags: 0,
            offset_middle: 0,
            offset_high: 0,
            _reserved: 0,
        }
    }

    /// Set the offset of this entry.
    pub fn set_offset(&mut self, offset: usize) {
        self.offset_low = (offset & 0xffff) as u16;
        self.offset_middle = ((offset >> 16) & 0xffff) as u16;
        self.offset_high = (offset >> 32) as u32
    }

    /// Set the segment selector for this entry.
    pub fn set_selector(&mut self, selector: u16) {
        self.selector = selector;
    }

    /// Set the interrupt stack table.
    pub fn set_ist(&mut self, ist: u8) {
        self.ist = ist & 0b111;
    }

    /// Mark the entry as present.
    pub fn set_p(&mut self, p: bool) {
        self.flags = (self.flags & !(1 << 7)) | ((p as u8) << 7);
    }

    /// Set the descriptor privilege level.
    pub fn set_dpl(&mut self, dpl: Ring) {
        self.flags = (self.flags & !(3 << 5)) | ((dpl as u8) << 5);
    }

    /// Set the type of this entry.
    pub fn set_type(&mut self, typ: GateType) {
        self.flags = (self.flags & !(0xf)) | typ as u8;
    }
}

pub unsafe fn set_entry(index: usize, handler: unsafe extern "C" fn()) {
    EARLY_IDT.entries[index].set_p(true);
    EARLY_IDT.entries[index].set_type(GateType::Trap);
    EARLY_IDT.entries[index].set_dpl(Ring::Ring3);
    EARLY_IDT.entries[index].set_selector(cs().bits());
    EARLY_IDT.entries[index].set_offset(handler as usize);
}

/// Initialise the early interrupt handlers.
pub unsafe fn init_early() {
    println!("Initialising early IDT");

    set_entry(0, traps::divide_by_zero);
    set_entry(1, traps::debug);
    set_entry(2, traps::nmi);
    set_entry(3, traps::breakpoint);
    set_entry(4, traps::overflow);
    set_entry(5, traps::bound_range);
    set_entry(6, traps::invalid_opcode);
    set_entry(7, traps::device_not_available);
    set_entry(8, traps::double_fault);
    //set_entry(9);
    set_entry(10, traps::invalid_tss);
    set_entry(11, traps::segment_not_present);
    set_entry(12, traps::stack);
    set_entry(13, traps::general_protection);
    set_entry(14, traps::page_fault);
    //set_entry(15);
    set_entry(16, traps::x87_floating_point_ex_pending);
    set_entry(17, traps::alignment_check);
    set_entry(18, traps::machine_check);
    set_entry(19, traps::simd_floating_point);
    //set_entry(20);
    set_entry(21, traps::control_protection);
    //set_entry(22);
    //set_entry(23);
    //set_entry(24);
    //set_entry(25);
    //set_entry(26);
    //set_entry(27);
    set_entry(28, traps::hypervisor_injection);
    set_entry(29, traps::vmm_communication);
    set_entry(30, traps::security);
    //set_entry(31);*/
    EARLY_IDT_PTR.base = EARLY_IDT.as_ptr();
    EARLY_IDT_PTR.limit = ((mem::size_of::<Entry>() * 32) - 1) as u16;

    x86::dtables::lidt(&EARLY_IDT_PTR);
    irq::enable();
}
