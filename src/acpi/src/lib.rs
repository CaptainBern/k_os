#![no_std]

use core::{mem, ptr, result, slice};

pub mod madt;
pub mod structs;

pub type Result<T> = result::Result<T, AcpiError>;

pub trait AcpiTable {
    const SIGNATURE: [u8; 4];
}

#[derive(Debug)]
pub enum AcpiError {
    InvalidSignature,
    UnsupportedRevision,
    ChecksumFailed,
}

/// Structure to keep track of ACPI tables.
#[derive(Debug)]
pub enum AcpiTables<'a> {
    Root(&'a structs::Rsdt),
    Extended(&'a structs::Xsdt),
}

impl<'a> AcpiTables<'a> {
    pub unsafe fn from_rsdt(addr: usize) -> Result<Self> {
        Ok(AcpiTables::Root(
            (addr as *const structs::Rsdt).as_ref().unwrap(),
        ))
    }

    pub unsafe fn from_xsdt() -> Result<Self> {
        todo!()
    }

    /// TODO: validation
    pub fn validate(&self) {}

    /// Iterate over the tables.
    pub fn iter(&self) -> Entries {
        Entries {
            tables: self,
            cur: 0,
        }
    }
}

#[derive(Debug)]
pub enum TableKind<'a> {
    Fadt(&'a structs::Fadt),
    Facs(&'a structs::Facs),
    Madt(&'a structs::Madt),
    Unknown(&'a structs::SdtHeader),
}

/// An iterator over RSDT/XSDT header entries.
pub struct Entries<'a> {
    /// Reference to the tables we're using.
    tables: &'a AcpiTables<'a>,

    /// Which entry are we currently parsing?
    cur: isize,
}

impl<'a> Entries<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        match self.tables {
            AcpiTables::Root(rsdt) => {
                (rsdt.header.length as usize - mem::size_of::<structs::SdtHeader>()) / 4
            }
            AcpiTables::Extended(xsdt) => {
                (xsdt.header.length as usize - mem::size_of::<structs::SdtHeader>()) / 8
            }
        }
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = TableKind<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur as usize >= self.len() {
            None
        } else {
            let header = match self.tables {
                AcpiTables::Root(ref rsdt) => unsafe {
                    let ptr = ptr::addr_of!(rsdt.entries)
                        .offset(self.cur)
                        .read_unaligned();
                    let raw = ptr[0] as *const structs::SdtHeader;
                    raw.as_ref().unwrap()
                },
                AcpiTables::Extended(ref xsdt) => unsafe {
                    let ptr = ptr::addr_of!(xsdt.entries)
                        .offset(self.cur)
                        .read_unaligned();
                    let raw = ptr[0] as *const structs::SdtHeader;
                    raw.as_ref().unwrap()
                },
            };

            // todo: validate the header

            self.cur += 1;

            unsafe {
                match header.signature {
                    structs::Facs::SIGNATURE => Some(TableKind::Facs(
                        (&header as *const _ as *const structs::Facs)
                            .as_ref()
                            .unwrap(),
                    )),
                    structs::Fadt::SIGNATURE => Some(TableKind::Fadt(
                        (&header as *const _ as *const structs::Fadt)
                            .as_ref()
                            .unwrap(),
                    )),
                    structs::Madt::SIGNATURE => Some(TableKind::Madt(
                        (&header as *const _ as *const structs::Madt)
                            .as_ref()
                            .unwrap(),
                    )),
                    _ => Some(TableKind::Unknown(&header)),
                }
            }
        }
    }
}

/// Calculate the checksum of a given object.
fn calc_checksum<T>(object: &T) -> u8 {
    let bytes =
        unsafe { slice::from_raw_parts((object as *const T) as *const u8, mem::size_of::<T>()) };

    bytes.iter().sum::<u8>()
}
