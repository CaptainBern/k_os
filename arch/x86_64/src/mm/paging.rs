use core::fmt::{Debug, Formatter, Result};

use bitflags::bitflags;

pub const KILOBYTE: usize = 1024;
pub const MEGABYTE: usize = 1024 * KILOBYTE;
pub const GIGABYTE: usize = 1024 * MEGABYTE;

pub const BASE_PAGE: usize = 4 * KILOBYTE;
pub const MEGA_PAGE: usize = 2 * MEGABYTE;
pub const GIGA_PAGE: usize = 1 * GIGABYTE;

pub const PT_COVERAGE: usize = 512 * BASE_PAGE;
pub const PD_COVERAGE: usize = 512 * MEGA_PAGE;
pub const PDPT_COVERAGE: usize = 512 * GIGA_PAGE;
pub const PML4_COVERAGE: usize = 512 * PDPT_COVERAGE;

pub const PML4_BIT_SHIFT: u64 = 39;
pub const PDPT_BIT_SHIFT: u64 = 30;
pub const PD_BIT_SHIFT: u64 = 21;
pub const PT_BIT_SHIFT: u64 = 12;

/// The maximum number of bits in a physical address.
pub const MAXPHYADDRESS: u64 = 52;

/// Maximum number of bits in a virtual address. (This is for 4-level paging)
pub const MAX_VADDR_BITS: u64 = 48;

/// Mask used to test if an address is in canonical form.
pub const CANONICAL_ADDRESS_MASK: u64 = !((1 << MAX_VADDR_BITS as u64 - 1) - 1);

/// Mask used to check if an address is page aligned.
pub const PAGE_ALIGN_MASK: u64 = (1 << PT_BIT_SHIFT) - 1;

/// Mask for table addresses.
pub const ADDRESS_MASK: u64 = ((1 << MAXPHYADDRESS) - 1) & !0xfff;

/// Return true if the given address is canonical.
#[inline]
pub const fn is_canonical(addr: u64) -> bool {
    (addr & CANONICAL_ADDRESS_MASK == CANONICAL_ADDRESS_MASK) | (addr & CANONICAL_ADDRESS_MASK == 0)
}

/// Compute the PML4 index of the given address.
#[inline]
pub const fn pml4_index(virt: u64) -> usize {
    ((virt >> PML4_BIT_SHIFT) & 0b111111111) as usize
}

/// Compute the PDPT index of the given address.
#[inline]
pub const fn pdpt_index(virt: u64) -> usize {
    ((virt >> PDPT_BIT_SHIFT) & 0b111111111) as usize
}

/// Compute the PD index of the given address.
#[inline]
pub const fn pd_index(virt: u64) -> usize {
    ((virt >> PD_BIT_SHIFT) & 0b111111111) as usize
}

/// Compute the frame number of the given address.
#[inline]
pub const fn pt_index(virt: u64) -> usize {
    ((virt >> PT_BIT_SHIFT) & 0b111111111) as usize
}

/// Returns true if `addr` is aligned on `ALIGNMENT`.
///
/// `ALIGNMENT` should be a power of two.
#[inline]
pub const fn is_aligned<const ALIGNMENT: usize>(addr: u64) -> bool {
    assert!(ALIGNMENT.is_power_of_two());
    addr & ((1 << ALIGNMENT.trailing_zeros()) - 1) == 0
}

/// Align `addr` down on `ALIGNMENT`.
///
/// `ALIGNMENT` should be a power of two.
#[inline]
pub const fn align_down<const ALIGNMENT: usize>(addr: u64) -> u64 {
    assert!(ALIGNMENT.is_power_of_two());
    addr & !((1 << ALIGNMENT.trailing_zeros()) - 1)
}

/// Align `addr` up on `ALIGNMENT`.
///
/// `ALIGNMENT` should be a power of two.
#[inline]
pub const fn align_up<const ALIGNMENT: usize>(addr: u64) -> u64 {
    assert!(ALIGNMENT.is_power_of_two());
    (addr + ALIGNMENT as u64 - 1) & !((1 << ALIGNMENT.trailing_zeros()) - 1)
}

/// Return how many tables (or frames) with `COVERAGE` are needed to map a memory block
/// of the given size.
#[inline]
pub const fn num_tables<const COVERAGE: usize>(size: usize) -> usize {
    align_up::<COVERAGE>(size as u64) as usize / COVERAGE
}

/// A PML4 table.
#[derive(Debug, Clone, Copy)]
#[repr(align(4096))]
pub struct PML4 {
    pub table: [PML4E; 512],
}

impl PML4 {
    pub const fn zero() -> Self {
        Self {
            table: [PML4E::ZERO; 512],
        }
    }
}

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
#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct PML4E {
    pub bits: u64,
}

impl PML4E {
    pub const ZERO: PML4E = PML4E { bits: 0 };

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

impl Debug for PML4E {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("PML4E")
            .field("address", &self.address())
            .field("flags", &self.flags())
            .finish()
    }
}

/// Page Directory Pointer Table.
#[derive(Debug, Clone, Copy)]
#[repr(align(4096))]
pub struct PDPT {
    pub table: [PDPTE; 512],
}

impl PDPT {
    pub const fn zero() -> Self {
        Self {
            table: [PDPTE::ZERO; 512],
        }
    }
}

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
#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct PDPTE {
    pub bits: u64,
}

impl PDPTE {
    pub const ZERO: PDPTE = PDPTE { bits: 0 };

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
        self.bits |= pd & ADDRESS_MASK;
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

impl Debug for PDPTE {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("PDPTE")
            .field("address", &self.address())
            .field("flags", &self.flags())
            .finish()
    }
}

/// Page Directory.
#[derive(Debug, Clone, Copy)]
#[repr(align(4096))]
pub struct PD {
    pub table: [PDE; 512],
}

impl PD {
    pub const fn zero() -> Self {
        Self {
            table: [PDE::ZERO; 512],
        }
    }
}

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
#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct PDE {
    pub bits: u64,
}

impl PDE {
    pub const ZERO: PDE = PDE { bits: 0 };

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

impl Debug for PDE {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("PDE")
            .field("address", &self.address())
            .field("flags", &self.flags())
            .finish()
    }
}

/// Page Table.
#[derive(Debug, Clone, Copy)]
#[repr(align(4096))]
pub struct PT {
    pub table: [PTE; 512],
}

impl PT {
    pub const fn zero() -> Self {
        Self {
            table: [PTE::ZERO; 512],
        }
    }
}

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
#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct PTE {
    pub bits: u64,
}

impl PTE {
    pub const ZERO: PTE = PTE { bits: 0 };

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

impl Debug for PTE {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("PTE")
            .field("address", &self.frame())
            .field("flags", &self.flags())
            .finish()
    }
}
