pub mod structs;

use crate::{sdt::SdtHeader, AcpiTable};
use bitflags::bitflags;
use core::mem;

pub use structs::*;

bitflags! {
    /// Multiple APIC Flags.
    ///
    /// See ACPI v6.4 table 5.20
    #[derive(Debug, Clone, Copy)]
    pub struct MaFlags: u32 {
        const PCAT_COMPAT = 1;
    }
}

/// Multiple APIC Description Table.
///
/// See ACPI v6.4 section 5.2.12
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Madt {
    pub header: SdtHeader,
    pub local_apic_address: u32,
    pub flags: MaFlags,
}

impl AcpiTable for Madt {
    const SIGNATURE: [u8; 4] = *b"APIC";
}

impl Madt {
    #[inline]
    pub fn iter(&self) -> Structures {
        Structures {
            madt: self,
            len: self.header.length as usize - mem::size_of::<Madt>(),
            cur: 0,
        }
    }
}

#[derive(Debug)]
pub enum ApicStructureKind<'a> {
    ProcessorLocalApic(&'a ProcessorLocalApicStructure),
    IoApic(&'a IoApicStructure),
    InterruptSourceOverrice(&'a IntSourceOverrideStructure),
    NmiSource(&'a NmiSourceStructure),
    LocalApicNmi(&'a LocalApicNmiStructure),
    LocalApicAddressOverride(&'a LocalApicAdressOverrideStructure),
    IoSapic(&'a IoSapicStructure),
    LocalSapic(&'a LocalSapicStructure),
    PlatformInterruptSource(&'a PlatformInterruptSourceStructure),
    ProcessorLocalX2Apic(&'a ProcessorLocalX2ApicStructure),
    LocalX2ApicNmi(&'a LocalX2ApicNmiStructure),
    Gicc(&'a GiccStructure),
    Gicd(&'a GicdStructure),
    GicMsiFrame(&'a GicMsiFrameStructure),
    Gicr(&'a GicrStructure),
    GicIts(&'a GicItsStructure),
    MultiprocessorWakeup(&'a MultiProcessorWakeupStructure),
    Reserved(&'a ApicStructureHeader),
    Oem(&'a ApicStructureHeader),
}

#[derive(Debug)]
pub struct Structures<'a> {
    madt: &'a Madt,
    len: usize,
    cur: isize,
}

impl<'a> Iterator for Structures<'a> {
    type Item = ApicStructureKind<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur as usize >= self.len {
            None
        } else {
            let header = unsafe {
                let base = (self.madt as *const _ as *const u8)
                    .offset(mem::size_of::<Madt>() as isize)
                    .offset(self.cur);
                (base as *const ApicStructureHeader).as_ref().unwrap()
            };

            self.cur += header.length as isize;

            macro_rules! match_structs {
                (
                    match $header:ident.$field:ident => {
                        $($id:literal => $raw:ty as $wrapper:path),*
                    }
                ) => {
                    match $header.$field {
                        $($id => Some($wrapper(
                            unsafe { ($header as *const _ as *const $raw).as_ref().unwrap() }
                        )),)*
                        0x11..=0x7f => Some(ApicStructureKind::Reserved($header)),
                        0x80..=0xff => Some(ApicStructureKind::Oem($header)),
                    }
                };
            }

            match_structs! {
                match header.entry_type => {
                    0 => ProcessorLocalApicStructure as ApicStructureKind::ProcessorLocalApic,
                    1 => IoApicStructure as ApicStructureKind::IoApic,
                    2 => IntSourceOverrideStructure as ApicStructureKind::InterruptSourceOverrice,
                    3 => NmiSourceStructure as ApicStructureKind::NmiSource,
                    4 => LocalApicNmiStructure as ApicStructureKind::LocalApicNmi,
                    5 => LocalApicAdressOverrideStructure as ApicStructureKind::LocalApicAddressOverride,
                    6 => IoSapicStructure as ApicStructureKind::IoSapic,
                    7 => LocalSapicStructure as ApicStructureKind::LocalSapic,
                    8 => PlatformInterruptSourceStructure as ApicStructureKind::PlatformInterruptSource,
                    9 => ProcessorLocalX2ApicStructure as ApicStructureKind::ProcessorLocalX2Apic,
                    10 => LocalX2ApicNmiStructure as ApicStructureKind::LocalX2ApicNmi,
                    11 => GiccStructure as ApicStructureKind::Gicc,
                    12 => GicdStructure as ApicStructureKind::Gicd,
                    13 => GicMsiFrameStructure as ApicStructureKind::GicMsiFrame,
                    14 => GicrStructure as ApicStructureKind::Gicr,
                    15 => GicItsStructure as ApicStructureKind::GicIts,
                    16 => MultiProcessorWakeupStructure as ApicStructureKind::MultiprocessorWakeup
                }
            }
        }
    }
}
