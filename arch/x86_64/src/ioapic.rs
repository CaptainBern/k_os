use self::registers::{
    Arbitration, IoApicId, RedirectionTableEntry, Version, IO_APIC_ARB_ID_REG, IO_APIC_ID_REG,
    IO_APIC_RED_TBL_0, IO_APIC_REG_SEL, IO_APIC_REG_WIN, IO_APIC_VERSION_REG,
};

pub mod registers;

/// Access to the IOAPIC.
#[derive(Debug)]
pub struct IoApic {
    sel: *mut u32,
    win: *mut u32,
}

impl IoApic {
    /// Construct a new IOAPIC with the given base.
    ///
    /// # Safety
    /// `base` must be a valid virtual address pointing to the base of the
    /// IOAPIC.
    pub const unsafe fn new(base: *mut u32) -> Self {
        Self {
            sel: base.byte_add(IO_APIC_REG_SEL as usize),
            win: base.byte_add(IO_APIC_REG_WIN as usize),
        }
    }

    /// Read the APIC ID register.
    ///
    /// The 4-bit ID contained in it serves as a physical name of the IOAPIC.
    pub fn id(&mut self) -> u8 {
        unsafe { IoApicId::from_bits_unchecked(self.unchecked_read(IO_APIC_ID_REG)).ioapic_id() }
    }

    /// Set the APIC ID.
    ///
    /// All APIC devices using the APIC bus should have a unique APIC ID. The
    /// APIC bus arbitration ID for the I/O unit is also written during a write
    /// to the APICID register (same data is loaded into both). This register
    /// must be programmed with the correct ID before using the IOAPIC for
    /// message transmission.
    pub fn set_id(&mut self, id: u8) {
        unsafe { self.unchecked_write(IO_APIC_ID_REG, IoApicId::new(id).bits()) }
    }

    /// Read the APIC Version register.
    ///
    /// This register can be used to provide compatibility between different
    /// APIC implementations and their versions. In addition, this field also
    /// provides the maximum number of entries in the I/O Redirection Table.
    pub fn version(&mut self) -> Version {
        unsafe { Version::from_bits_unchecked(self.unchecked_read(IO_APIC_VERSION_REG)) }
    }

    /// Read the APIC Arbitration register.
    ///
    /// This register contains the bus arbitration priority for the IOAPIC.
    /// This register is loaded when when the IOAPIC ID register is written.
    pub fn arbitration(&mut self) -> Arbitration {
        unsafe { Arbitration::from_bits_unchecked(self.unchecked_read(IO_APIC_ARB_ID_REG)) }
    }

    /// Mask all redirection entries.
    pub fn mask_all(&mut self) {
        for i in 0..self.version().max_redir_entry() {
            let mut entry = self.redirection_entry(i);
            entry.low.set_masked(true);
            self.set_redirection_entry(i, entry);
        }
    }

    /// Read an entry from the Redirection Table.
    ///
    /// There are 24 entries in the I/O Redirection Table, indexed at 0.
    /// Each entry is a dedicated entry for each interrupt input signal. Unlike
    /// IRQ pins of the 8259A, the notion of interrupt priority is entirely
    /// unrelated to the position of the physical interrupt inpu signal on the
    /// APIC. Instead, software determines the vector (and therefore the
    /// priority) for each corresponding interrupt input signal. The
    /// information in the Redirection Table is used to translate the
    /// corresponding interrupt pin information into an inter-APIC message.
    pub fn redirection_entry(&mut self, idx: u8) -> RedirectionTableEntry {
        assert!(idx <= 23);
        let idx = idx as u32 * IO_APIC_RED_TBL_0;
        unsafe {
            let low = self.unchecked_read(idx);
            let high = self.unchecked_read(idx + 2);
            RedirectionTableEntry::from_bits_unchecked(low, high)
        }
    }

    /// Write an entry in the Redirection Table.
    ///
    /// See [`redirection_entry`] for more info.
    pub fn set_redirection_entry(&mut self, idx: u8, entry: RedirectionTableEntry) {
        assert!(idx <= 23);
        let idx = idx as u32 * IO_APIC_RED_TBL_0;
        unsafe {
            self.unchecked_write(idx, entry.low.bits());
            self.unchecked_write(idx + 2, entry.high.bits());
        }
    }

    /// Perform an unchecked read on the IOAPIC.
    ///
    /// # Safety
    /// It is up to the caller to make sure `reg` is a valid IOAPIC register.
    pub unsafe fn unchecked_read(&mut self, reg: u32) -> u32 {
        self.sel.write_volatile(reg);
        self.win.read_volatile()
    }

    /// Perform an unchecked write to the IOAPIC.
    ///
    /// # Safety
    /// It is up to the caller that `reg` and `val` are legal values.
    pub unsafe fn unchecked_write(&mut self, reg: u32, val: u32) {
        self.sel.write_volatile(reg);
        self.win.write_volatile(val);
    }
}
