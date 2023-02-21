use bitflags::bitflags;

/// The maximum number of bits in a physical address.
pub const MAXPHYADDRESS: u64 = 52;

const ADDRESS_MASK: u64 = !((1 << MAXPHYADDRESS) - 1) & !0xfff;

/// A PML4 table.
pub type PML4 = [PML4E; 512];

bitflags! {
    #[repr(transparent)]
    pub struct PML4EFlags: u64 {
        /// Present; must be 1 to reference a PDPT.
        const P = 1 << 0;

        /// R/W; if 0, writes may not be allowed to the 512GByte region controlled by this entry.
        const RW = 1 << 1;

        /// U/S; if 0, user-mode accesses are not allowed to the 512GByte region controlled by this entry.
        const US = 1 << 2;

        /// Page-level Write-Through; indirectly determines the memory type used to access the PDP-table referenced
        /// by this entry.
        const PWT = 1 << 3;

        /// Page-level Cache Disable; indirectly determines the memory type used to access the PDP-table referenced
        /// by this entry.
        const PCD = 1 << 4;

        /// Accessed; indicates whether this entry has been used for linear-address translation.
        const A = 1 << 5;

        /// Ignored by hardware.
        const IGNORED_0 = 1 << 6;

        /// Reserved; must be 0.
        const PS = 1 << 7;

        /// Available to user.
        const USER_0 = 1 << 8;

        /// Available to user.
        const USER_1 = 1 << 9;

        /// Available to user.
        const USER_2 = 1 << 10;

        /// For ordinary paging ignored, for HLAT paging, restart.
        const R = 1 << 11;

        /// If IA32_EFER.NXE = 1; execute-disable. Otherwise must be 0.
        const XD = 1 << 63;
    }
}

/// PML4 Entry.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PML4E {
    pub bits: u64,
}

impl PML4E {
    pub const NULL: PML4E = PML4E { bits: 0 };

    /// Initialise a new PML4 entry.
    ///
    /// The entry must refer to a [PDPT] if [PML4EFlags::P] is set.
    ///
    /// * `pdpt` - The physical address of the PDPT.
    /// * `flags` - The flags of the PML4 entry.
    #[inline]
    pub const fn new(pdpt: u64, flags: PML4EFlags) -> Self {
        PML4E {
            bits: (pdpt & ADDRESS_MASK) | flags.bits,
        }
    }

    #[inline]
    pub const fn address(&self) -> u64 {
        self.bits & ADDRESS_MASK
    }

    #[inline]
    pub fn set_address(&mut self, pdpt: u64) {
        self.bits &= !ADDRESS_MASK;
        self.bits |= pdpt & ADDRESS_MASK;
    }

    #[inline]
    pub const fn flags(&self) -> PML4EFlags {
        PML4EFlags::from_bits_truncate(self.bits & !ADDRESS_MASK)
    }

    #[inline]
    pub fn set_flags(&mut self, flags: PML4EFlags) {
        self.bits &= ADDRESS_MASK;
        self.bits |= flags.bits;
    }
}

/// Page Directory Pointer Table.
pub type PDPT = [PDPTE; 512];

bitflags! {
    #[repr(transparent)]
    pub struct PDPTEFlags: u64 {
        /// Present; must be 1 to reference a PD or a 1GByte page.
        const P = 1 << 0;

        /// R/W; if 0, writes may not be allowed to the 1GByte region controlled by this entry.
        const RW = 1 << 1;

        /// U/S; if 0, user-mode accesses are not allowed to the 1GByte region controlled by this entry.
        const US = 1 << 2;

        /// Page-level Write-Through; indirectly determines the memory type used to access the PD referenced
        /// by this entry.
        const PWT = 1 << 3;

        /// Page-level Cache Disable; indirectly determines the memory type used to access the PD referenced
        /// by this entry.
        const PCD = 1 << 4;

        /// Accessed; indicates whether this entry has been used for linear-address translation.
        const A = 1 << 5;

        /// Ignored by hardware.
        const IGNORED_0 = 1 << 6;

        /// Page size; when set this entry references a 1Gbyte page.
        const PS = 1 << 7;

        /// Ignored by hardware.
        const IGNORED_1 = 1 << 8;

        /// Available to user.
        const USER_0 = 1 << 9;

        /// Available to user.
        const USER_1 = 1 << 10;

        /// Available to user.
        const USER_2 = 1 << 11;

        /// If IA32_EFER.NXE = 1; execute-disable. Otherwise must be 0.
        const XD = 1 << 63;
    }
}

/// Page Directory Pointer Table Entry.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PDPTE {
    pub bits: u64,
}

impl PDPTE {
    pub const NULL: PDPTE = PDPTE { bits: 0 };

    /// Initialise a PDPT entry.
    ///
    /// This entry can either refer to a [PD] or a 1-GByte page (if supported by the CPU).
    ///
    /// * `pd` - The physical address of the [PD] in case the [PDPTEFlags::P] bit is set
    ///     If the [PDPTEFlags::PS] bit is set the entry refers to a 1-GByte page, in which
    ///     case `pd` is the frame number.
    /// * `flags` - The flags for the PDPT entry.
    #[inline]
    pub const fn new(pd: u64, flags: PDPTEFlags) -> Self {
        PDPTE {
            bits: (pd & ADDRESS_MASK) | flags.bits,
        }
    }

    #[inline]
    pub const fn address(&self) -> u64 {
        self.bits & ADDRESS_MASK
    }

    #[inline]
    pub fn set_address(&mut self, pd: u64) {
        self.bits &= !ADDRESS_MASK;
        self.bits &= pd & ADDRESS_MASK;
    }

    #[inline]
    pub const fn flags(&self) -> PDPTEFlags {
        PDPTEFlags::from_bits_truncate(self.bits & !ADDRESS_MASK)
    }

    #[inline]
    pub fn set_flags(&mut self, flags: PDPTEFlags) {
        self.bits &= ADDRESS_MASK;
        self.bits |= flags.bits;
    }
}

/// Page Directory.
pub type PD = [PDE; 512];

bitflags! {
    #[repr(transparent)]
    pub struct PDEFlags: u64 {
        /// Present; must be 1 to reference a PT or a 2MByte page.
        const P = 1 << 0;

        /// R/W; if 0, writes may not be allowed to the 2MByte region controlled by this entry.
        const RW = 1 << 1;

        /// U/S; if 0, user-mode accesses are not allowed to the 2MByte region controlled by this entry.
        const US = 1 << 2;

        /// Page-level Write-Through; indirectly determines the memory type used to access the PT referenced
        /// by this entry.
        const PWT = 1 << 3;

        /// Page-level Cache Disable; indirectly determines the memory type used to access the PT referenced
        /// by this entry.
        const PCD = 1 << 4;

        /// Accessed; indicates whether this entry has been used for linear-address translation.
        const A = 1 << 5;

        /// Ignored by hardware.
        const IGNORED_0 = 1 << 6;

        /// Page size; when set this entry references a 2MByte page.
        const PS = 1 << 7;

        /// Ignored by hardware.
        const IGNORED_1 = 1 << 8;

        /// Available to user.
        const USER_0 = 1 << 9;

        /// Available to user.
        const USER_1 = 1 << 10;

        /// Available to user.
        const USER_2 = 1 << 11;

        /// If IA32_EFER.NXE = 1; execute-disable. Otherwise must be 0.
        const XD = 1 << 63;
    }
}

/// Page Directory Entry.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PDE {
    pub bits: u64,
}

impl PDE {
    pub const NULL: PDE = PDE { bits: 0 };

    /// Initialise a new PD entry.
    ///
    /// This entry can either refer to a [PT] or a 2-MByte page.
    ///
    /// * `pt` - The physical address of the [PT] in case the [PDEFlags::P] bit is set.
    ///     If the [PDEFlags::PS] bit is set, the entry refers to a 2-MByte page, in which
    ///     case `pt` refers to the frame number.
    /// * `flags` - The flags for this entry.
    #[inline]
    pub const fn new(pt: u64, flags: PDEFlags) -> Self {
        PDE {
            bits: (pt & ADDRESS_MASK) | flags.bits,
        }
    }

    #[inline]
    pub const fn address(&self) -> u64 {
        self.bits & ADDRESS_MASK
    }

    #[inline]
    pub fn set_address(&mut self, pt: u64) {
        self.bits &= !ADDRESS_MASK;
        self.bits |= pt & ADDRESS_MASK;
    }

    #[inline]
    pub const fn flags(&self) -> PDEFlags {
        PDEFlags::from_bits_truncate(self.bits & !ADDRESS_MASK)
    }

    #[inline]
    pub fn set_flags(&mut self, flags: PDEFlags) {
        self.bits &= ADDRESS_MASK;
        self.bits |= flags.bits;
    }
}

/// Page Table.
pub type PT = [PTE; 512];

bitflags! {
    #[repr(transparent)]
    pub struct PTEFlags: u64 {
        /// Present; must be 1 to map a 4KByte page.
        const P = 1 << 0;

        /// R/W; if 0, writes may not be allowed to the 4KByte page referenced by this entry.
        const RW = 1 << 1;

        /// U/S; if 0, user-mode accesses are not allowed to the 4KByte page referenced by this entry.
        const US = 1 << 2;

        /// Page-level Write-Through; indirectly determines the memory type used to access the 4KByte page referenced
        /// by this entry.
        const PWT = 1 << 3;

        /// Page-level Cache Disable; indirectly determines the memory type used to access the 4KByte page referenced
        /// by this entry.
        const PCD = 1 << 4;

        /// Accessed; indicates whether software has accessed the 4KByte page referenced by this entry.
        const A = 1 << 5;

        /// Dirty; indicates whether software has written to the 4KByte page referenced by this entry.
        const D = 1 << 6;

        /// Indirectly determines the memory type used to access the 4KByte page referenced by this entry.
        const PAT = 1 << 7;

        /// Global; if CR4.PGE = 1, determines whether the translation is global.
        const G = 1 << 8;

        /// Available to user.
        const USER_0 = 1 << 9;

        /// Available to user.
        const USER_1 = 1 << 10;

        /// Available to user.
        const USER_2 = 1 << 11;

        /// Available to user.
        const USER_3 = 1 << 52;

        /// Available to user.
        const USER_4 = 1 << 53;

        /// Available to user.
        const USER_5 = 1 << 54;

        /// Available to user.
        const USER_6 = 1 << 55;

        /// Available to user.
        const USER_7 = 1 << 56;

        /// Available to user.
        const USER_8 = 1 << 57;

        /// Available to user.
        const USER_9 = 1 << 58;

        /// If IA32_EFER.NXE = 1; execute-disable. Otherwise must be 0.
        const XD = 1 << 63;
    }
}

/// Page Table Entry.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PTE {
    pub bits: u64,
}

impl PTE {
    pub const NULL: PTE = PTE { bits: 0 };

    /// Initialise a new PT entry.
    ///
    /// This entry refers to a 4-KByte page.
    ///
    /// * `frame` - The frame number in case [PTEFlags::P] bit is set.
    /// * `flags` - The flags for this entry.
    #[inline]
    pub const fn new(frame: u64, flags: PTEFlags) -> Self {
        PTE {
            bits: (frame & ADDRESS_MASK) | flags.bits,
        }
    }

    #[inline]
    pub const fn frame(&self) -> u64 {
        self.bits & ADDRESS_MASK
    }

    #[inline]
    pub fn set_frame(&mut self, frame: u64) {
        self.bits &= !ADDRESS_MASK;
        self.bits |= frame & ADDRESS_MASK;
    }

    #[inline]
    pub const fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.bits & !ADDRESS_MASK)
    }

    #[inline]
    pub fn set_flags(&mut self, flags: PTEFlags) {
        self.bits &= ADDRESS_MASK;
        self.bits |= flags.bits
    }
}
