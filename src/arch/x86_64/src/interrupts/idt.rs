use core::{
    ops::{Deref, DerefMut},
    ptr::{addr_of, addr_of_mut, read_unaligned, write_unaligned},
};

use x86::{segmentation::SegmentSelector, Ring};

/// 64-bit gate descriptor type.
#[repr(u8)]
pub enum GateType {
    Interrupt = 0b1110,
    Trap = 0b1111,
}

/// 64-bit gate descriptor options.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct GateOptions(u16);

impl GateOptions {
    pub const fn new() -> GateOptions {
        GateOptions(0)
    }

    /// Toggle the present bit.
    pub fn set_p(&mut self, p: bool) {
        self.0 = (self.0 & !(1 << 15)) | ((p as u16) << 15);
    }

    /// Return true if the 'P' bit is set.
    pub fn p(&self) -> bool {
        self.0 & (1 << 15) == (1 << 15)
    }

    /// Set the descriptor privilege level.
    pub fn set_dpl(&mut self, ring: Ring) {
        self.0 = (self.0 & !(0b11 << 13)) | ((ring as u16) << 13);
    }

    /// Return the DPL.
    pub fn dpl(&self) -> Ring {
        match (self.0 & (0b11 << 13)) >> 13 {
            0 => Ring::Ring0,
            1 => Ring::Ring1,
            2 => Ring::Ring2,
            3 => Ring::Ring3,
            _ => unreachable!("Malformed DPL"),
        }
    }

    /// Set the descriptor type.
    pub fn set_type(&mut self, typ: GateType) {
        self.0 = (self.0 & !(0b1111 << 8)) | ((typ as u16) << 8);
    }

    /// Return the descriptor type.
    pub fn typ(&self) -> GateType {
        match (self.0 & (0b1111 << 8)) >> 8 {
            0b1110 => GateType::Interrupt,
            0b1111 => GateType::Trap,
            _ => unreachable!("Malformed gate type"),
        }
    }

    /// Set the interrupt stack table.
    pub fn set_ist(&mut self, ist: u8) {
        self.0 = (self.0 & !(0b111)) | ((ist as u16) & 0b111);
    }

    /// Return the IST.
    pub fn ist(&self) -> u8 {
        (self.0 & 0b111) as u8
    }
}

/// A guard providing (easier) mutable access to
/// GateOptions.
pub struct GateOptionsGuard<'a> {
    descriptor: &'a mut GateDescriptor,
    copy: GateOptions,
}

impl<'a> Deref for GateOptionsGuard<'a> {
    type Target = GateOptions;

    fn deref(&self) -> &Self::Target {
        &self.copy
    }
}

impl<'a> DerefMut for GateOptionsGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.copy
    }
}

impl<'a> Drop for GateOptionsGuard<'a> {
    fn drop(&mut self) {
        let unaligned = addr_of_mut!(self.descriptor.options);
        unsafe {
            write_unaligned(unaligned, self.copy);
        }
    }
}

/// A 64-bit gate descriptor.
#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct GateDescriptor {
    offset_low: u16,
    selector: u16,
    options: GateOptions,
    offset_middle: u16,
    offset_high: u32,
    _reserved: u32,
}

impl GateDescriptor {
    pub const fn new() -> GateDescriptor {
        GateDescriptor {
            offset_low: 0,
            selector: 0,
            options: GateOptions::new(),
            offset_middle: 0,
            offset_high: 0,
            _reserved: 0,
        }
    }

    /// Set the entrypoint to the ISR for this descriptor.
    pub fn set_offset(&mut self, offset: usize) {
        self.offset_low = (offset & 0xffff) as u16;
        self.offset_middle = ((offset >> 16) & 0xffff) as u16;
        self.offset_high = (offset >> 32) as u32;
    }

    /// Return the offset for this descriptor.
    pub fn offset(&self) -> usize {
        (self.offset_high as usize) << 32
            | (self.offset_middle as usize) << 16
            | self.offset_low as usize
    }

    /// Set the code segment for this descriptor.
    pub fn set_selector(&mut self, selector: SegmentSelector) {
        self.selector = selector.bits();
    }

    /// Return the code selector for this descriptor.
    pub fn selector(&self) -> SegmentSelector {
        SegmentSelector::from_bits_truncate(self.selector)
    }

    /// Set the options for this descriptor.
    pub fn set_options(&mut self, options: GateOptions) {
        self.options = options;
    }

    /// Return the options for this descriptor.
    pub fn options(&self) -> GateOptions {
        self.options
    }

    /// Return a guard for (easy) mutable access to the options.
    pub fn options_mut(&mut self) -> GateOptionsGuard {
        let unaligned = addr_of!(self.options);
        let copy = unsafe { read_unaligned(unaligned) };

        GateOptionsGuard {
            descriptor: self,
            copy: copy,
        }
    }
}
