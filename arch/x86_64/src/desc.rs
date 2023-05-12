//! x86_64 still uses segment descriptors. The interpretation of the descriptor
//! fields is changed and in some cases the descriptor itself is expanded. This
//! implementation is meant for dealing with 'longmode' descriptors
//! specifically. For more information refer to:
//!  - AMD64 Architecture Programmer's Manual Vol. 2 Chapters 4 and 8.
//!  - Intel Software Developer's Manual Vol. 3 Chapters 5, 6, and 7.

use bitflags::bitflags;
use x86::segmentation::SegmentSelector;

bitflags! {
    /// Descriptor access byte. The access byte as documented in the manuals is an
    /// actual full 8-bit byte, since it also contains the descriptor type. In this
    /// implementation we're treating the type as a seperate value. (See
    /// [UserDescriptorType], [SystemDescriptorType] and [GateDescriptorType])
    #[repr(transparent)]
    pub struct Access: u8 {
        /// When set, the descriptor refers to a user segment.
        const S = 1 << 0;

        /// Ring 0 privilege level for the segment the descriptor refers to.
        const DPL_0 = 0 * (1 << 1);

        /// Ring 1 privilege level for the segment the descriptor refers to.
        const DPL_1 = 1 * (1 << 1);

        /// Ring 2 privilege level for the segment the descriptor refers to.
        const DPL_2 = 2 * (1 << 1);

        /// Ring 3 privilege level for the segment the descriptor refers to.
        const DPL_3 = 3 * (1 << 1);

        /// Mark the segment referenced by this descriptor available.
        const P = 1 << 3;

        /// Access bits for a kernel user-segment descriptor.
        const KERNEL_USR = Access::S.bits | Access::DPL_0.bits | Access::P.bits;

        /// Access bits for a kernel system-segment descritptor.
        const KERNEL_SYS = Access::DPL_0.bits | Access::P.bits;
    }

    /// Possible flags for user- and system-descriptor. For system-descriptors,
    /// only G and AVL can be used, the other flags are expected to be zero.
    #[repr(transparent)]
    pub struct DescriptorFlags: u8 {
        /// Available to software.
        const AVL = 1 << 0;

        /// Only valid for code segment descriptors. When set, it specifies the processor
        /// is running in 64-bit mode. `DB` should be zero if this bit is set.
        const L = 1 << 1;

        /// Unused in longmode for data segment descriptors. Should be zero for code segment
        /// descriptors (if L=1) and system segment descriptors.
        const DB = 1 << 2;

        /// Ignored in longmode for both code- and data segment descriptors.
        const G = 1 << 3;
    }

    /// Type bits for a data segment descriptor.
    #[repr(transparent)]
    pub struct DataSegmentBits: u8 {
        /// Set by the processor when the descriptor is copied from the GDT or LDT into
        /// one of the data-segment registers or the stack-segment register.
        /// This bit is only cleared by software.
        const ACCESSED = 1 << 0;

        /// A read-only data segment.
        const READ_ONLY = 0 * (1 << 1);

        /// When set, the data-segment becomes writable. This bit is ignored in longmode,
        /// as read-write permissions are handled with paging.
        const READ_WRITE = 1 * (1 << 1);

        /// Read-only expand-down data segment. This bit is ignored in longmode.
        const READ_ONLY_EXP_D = 2 * (1 << 1);

        /// Read-write expand-down data segment. This bit is ignored in longmode.
        const READ_WRITE_EXP_D = 3 * (1 << 1);
    }

    /// Type bits for a code segment descriptor.
    #[repr(transparent)]
    pub struct CodeSegmentBits: u8 {
        /// Set by the processor when the descriptor is copied from the GDT or LDT into
        /// the `cs` register.
        /// This bit is only cleared by software.
        const ACCESSED = 1 << 0;

        /// Execute-only code-segment.
        const EX_ONLY = 4 * (1 << 1);

        /// Marks the code-segment as both executable and readable as data.
        /// When unset, attempting to read data from the code segment cause a general-
        /// protection exception. This bit is ignored in longmode.
        const EX_READ = 5 * (1 << 1);

        /// A conforming code-segment. When control is transferred to a higher-privilege
        /// conforming code segment from a lower-privilege code segment, the processor
        /// CPL does not change. Transfers to non-conforming code-segments with a higher
        /// privilege level than the CPL can only occur through gate descriptors.
        const EX_ONLY_CONF = 6 * (1 << 1);

        /// A conforming readable code-segment. The readable bit is ignored in longmode.
        /// See [DataSegmentBits::EX_ONLY_CONF].
        const EX_READ_CONF = 7 * (1 << 1);
    }
}

/// User descriptors come in two types: either code or data. The meaning of the
/// type bits change depending on whether the descriptor is for a code or data
/// segment. When using this type, the 'S' bit should be set on [Access].
#[derive(Debug, Clone, Copy)]
pub enum UserDescriptorType {
    Code(CodeSegmentBits),
    Data(DataSegmentBits),
}

impl UserDescriptorType {
    #[inline]
    pub const fn bits(&self) -> u8 {
        match self {
            UserDescriptorType::Code(code) => code.bits,
            UserDescriptorType::Data(data) => data.bits,
        }
    }
}

/// System descriptor types.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SystemDescriptorType {
    Ldt = 0x2,
    Tss = 0x9,
    BusyTss = 0xb,
}

impl SystemDescriptorType {
    #[inline]
    pub const fn bits(&self) -> u8 {
        *self as u8
    }
}

/// Gate descriptor types.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum GateDescriptorType {
    Call = 0xc,
    Interrupt = 0xe,
    Trap = 0xf,
}

impl GateDescriptorType {
    #[inline]
    pub const fn bits(&self) -> u8 {
        *self as u8
    }
}

macro_rules! generic_descriptor {
    (
        $(#[$($meta:tt)*])*
        $descriptor:ident, $descriptor_type:ty, $bits:ty, $base:ty
    ) => {
        $(
            #[$($meta)*]
        )*
        #[derive(Debug, Clone, Copy)]
        #[repr(transparent)]
        pub struct $descriptor {
            pub bits: $bits,
        }

        impl $descriptor {
            #[inline]
            pub const fn new(
                base: $base,
                limit: u32,
                typ: $descriptor_type,
                access: Access,
                flags: DescriptorFlags,
            ) -> Self {
                let bits = if <$base>::BITS == 64 {
                        ((base as $bits) & 0xffffffff00000000) << 32
                    } else {
                        0
                    }
                    | ((base as $bits) & 0xff000000) << 24
                    | ((flags.bits() as $bits) & 0xf) << 52
                    | ((limit as $bits) & 0xf0000) << 32
                    | ((access.bits() as $bits) & 0xf) << 44
                    | ((typ.bits() as $bits) & 0xf) << 40
                    | ((base as $bits) & 0xffffff) << 16
                    | (limit as $bits) & 0xffff;
                $descriptor { bits: bits }
            }

            #[inline]
            pub fn set_limit(&mut self, limit: u32) {
                self.bits &= !(0xf00000000ffff);
                self.bits |= ((limit as $bits) & 0xf0000) << 32 | (limit as $bits) & 0xffff;
            }

            #[inline]
            pub fn set_base(&mut self, base: $base) {
                self.bits &=!(
                    if <$base>::BITS == 64 {
                        0xffffffff00000000 << 32
                    } else {
                        0
                    } | 0xff0000ffffff0000
                );
                self.bits |= if <$base>::BITS == 64 {
                        ((base as $bits) & 0xffffffff00000000) << 32
                    } else {
                        0
                    } | ((base as $bits) & 0xff000000) << 24 | ((base as $bits) & 0xffffff) << 16;
            }

            #[inline]
            pub fn set_flags(&mut self, flags: DescriptorFlags) {
                self.bits &= !(0xf0000000000000);
                self.bits |= ((flags.bits() as $bits) & 0xf) << 52;
            }
        }
    };
}

macro_rules! impl_descriptor_common {
    ($descriptor:ty, $typ:ty, $bits:ty) => {
        impl $descriptor {
            #[inline]
            pub fn set_type(&mut self, typ: $typ) {
                self.bits &= !(0xf0000000000);
                self.bits |= ((typ.bits() as $bits) & 0xf) << 40;
            }

            #[inline]
            pub fn set_access(&mut self, access: Access) {
                self.bits &= !(0xf00000000000);
                self.bits |= ((access.bits() as $bits) & 0xf) << 44;
            }
        }
    };
}

generic_descriptor!(
    #[doc = "An 8-byte user-segment descriptor."]
    UserDescriptor,
    UserDescriptorType,
    u64,
    u32
);
impl_descriptor_common!(UserDescriptor, UserDescriptorType, u64);

impl UserDescriptor {
    pub const NULL: UserDescriptor = UserDescriptor { bits: 0 };
}

generic_descriptor!(
    #[doc = "A 16-byte system-segment descriptor."]
    SystemDescriptor,
    SystemDescriptorType,
    u128,
    u64
);
impl_descriptor_common!(SystemDescriptor, SystemDescriptorType, u128);

impl SystemDescriptor {
    pub const NULL: SystemDescriptor = SystemDescriptor { bits: 0 };
}

/// A 16-byte gate descriptor.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct GateDescriptor {
    pub bits: u128,
}

impl GateDescriptor {
    pub const NULL: GateDescriptor = GateDescriptor { bits: 0 };

    #[inline]
    pub const fn new(
        offset: u64,
        selector: SegmentSelector,
        typ: GateDescriptorType,
        access: Access,
        ist: u8,
    ) -> Self {
        let bits = ((offset as u128) & 0xffffffffffff0000) << 32
            | ((access.bits() as u128) & 0xf) << 44
            | ((typ.bits() as u128) & 0xf) << 40
            | ((ist as u128) & 0b111) << 32
            | ((selector.bits() as u128) & 0xffff) << 16
            | (offset as u128) & 0xffff;
        GateDescriptor { bits }
    }

    #[inline]
    pub fn set_offset(&mut self, offset: u64) {
        self.bits &= !(0xffffffffffff00000000ffff);
        self.bits |= ((offset as u128) & 0xffffffffffff0000) << 32 | (offset as u128) & 0xffff;
    }

    #[inline]
    pub fn set_selector(&mut self, selector: SegmentSelector) {
        self.bits &= !(0xffff0000);
        self.bits |= ((selector.bits() as u128) & 0xffff) << 16
    }

    #[inline]
    pub fn set_ist(&mut self, ist: u8) {
        self.bits &= !(0b111 << 32);
        self.bits |= ((ist as u128) & 0b111) << 32;
    }
}

impl_descriptor_common!(GateDescriptor, GateDescriptorType, u128);

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct Tss {
    pub _reserved0: [u8; 4],
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    pub _reserved1: [u8; 4],
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    pub _reserved2: [u8; 6],
    pub io_map_base: u16,
}

impl Tss {
    #[inline]
    pub const fn new(rsp0: u64, io_map_base: u16) -> Tss {
        Tss {
            _reserved0: [0; 4],
            rsp0,
            rsp1: 0,
            rsp2: 0,
            _reserved1: [0; 4],
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            _reserved2: [0; 6],
            io_map_base,
        }
    }
}
