//! Small ACPI implementation for Rust.
//!
//! This crate provides the bare minimum to be able to work with ACPI in Rust.
//! The implementation is based on ACPI v6.4, as described [here](https://uefi.org/htmlspecs/ACPI_Spec_6_4_html/index.html).
//!
//! To keep the code as simple as possible, the caller is responsible for
//! making sure all the ACPI structures are reachable at their specified
//! address (there is no runtime 'remapping'). This essentially means it
//! assumes the ACPI memory region is identity mapped.

#![no_std]

use core::{mem, result};

pub mod address;
pub mod madt;
pub mod sdt;

pub type Result<T> = result::Result<T, AcpiError>;

#[derive(Debug)]
pub enum AcpiError {
    InvalidSignature,
    UnsupportedRevision,
    ChecksumFailed,
}

/// An ACPI table type.
pub trait AcpiTable {
    const SIGNATURE: [u8; 4];
}

/// Root System Description Table.
///
/// The array of entries are physical pointers to various other system description
/// tables.
/// See ACPI v6.4 section 5.2.7
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Rsdt {
    pub header: sdt::SdtHeader,
}

impl AcpiTable for Rsdt {
    const SIGNATURE: [u8; 4] = *b"RSDT";
}

/// Extended System Description Table.
///
/// Functionaly identical to [Rsdt], but instead using 64-bit pointers.
/// See ACPI v6.4 section 5.2.8
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Xsdt {
    pub header: sdt::SdtHeader,
}

impl AcpiTable for Xsdt {
    const SIGNATURE: [u8; 4] = *b"XSDT";
}

/// Enum wrapper for ACPI tables.
#[derive(Debug)]
pub enum AcpiTables<'a> {
    Root(&'a Rsdt),
    Extended(&'a Xsdt),
}

impl<'a> AcpiTables<'a> {
    /// Initialise the ACPI tables for a given RSDT address.
    ///
    /// # Safety
    /// The given address should point to a valid RSDT. The header is validated
    /// (which means its signature and checksum are checked) before it is
    /// accepted. However, a given address that points to a _valid_ RSDT
    /// containing bogus data can still cause unexpected behaviour.
    pub unsafe fn from_rsdt(addr: usize) -> Result<Self> {
        let rsdt = (addr as *const Rsdt).as_ref().unwrap();

        // Validate the header.
        rsdt.header.validate()?;

        // Check if the signature is correct.
        if rsdt.header.signature != Rsdt::SIGNATURE {
            Err(AcpiError::InvalidSignature)?
        }

        Ok(AcpiTables::Root(rsdt))
    }

    /// Initialise the ACPI tables for a given XSDT address.
    ///
    /// # Safety
    /// The given address should point to a valid XSDT. The header is validated
    /// (which means its signature and checksum are checked) before it is
    /// accepted. However, a given address that points to a _valid_ XSDT
    /// containing bogus data can still cause unexpected behaviour.
    pub unsafe fn from_xsdt(addr: usize) -> Result<Self> {
        let xsdt = (addr as *const Xsdt).as_ref().unwrap();

        // Validate the header.
        xsdt.header.validate()?;

        // Check if the signature is correct.
        if xsdt.header.signature != Xsdt::SIGNATURE {
            Err(AcpiError::InvalidSignature)?
        }

        Ok(AcpiTables::Extended(xsdt))
    }

    /// Compute the number of entries in the underlying table.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            AcpiTables::Root(rsdt) => {
                (rsdt.header.length as usize)
                    .checked_sub(mem::size_of::<sdt::SdtHeader>())
                    .unwrap_or(0)
                    / 4
            }
            AcpiTables::Extended(xsdt) => {
                (xsdt.header.length as usize)
                    .checked_sub(mem::size_of::<sdt::SdtHeader>())
                    .unwrap_or(0)
                    / 8
            }
        }
    }

    /// Return an iterator over the entries in the table.
    pub fn iter(&self) -> Entries {
        Entries {
            tables: self,
            len: self.len(),
            cur: 0,
        }
    }
}

#[derive(Debug)]
pub enum TableKind<'a> {
    Madt(&'a madt::Madt),
    Unknown(&'a sdt::SdtHeader),
}

impl<'a> TableKind<'a> {
    #[inline]
    pub fn header(&self) -> &'a sdt::SdtHeader {
        match self {
            TableKind::Madt(madt) => &madt.header,
            TableKind::Unknown(header) => &header,
        }
    }
}

/// An iterator over RSDT/XSDT header entries.
pub struct Entries<'a> {
    tables: &'a AcpiTables<'a>,
    len: usize,
    cur: isize,
}

impl<'a> Iterator for Entries<'a> {
    type Item = TableKind<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur as usize >= self.len {
            None
        } else {
            let header = match self.tables {
                AcpiTables::Root(ref rsdt) => unsafe {
                    let ptr = (*rsdt as *const _ as *const u8)
                        .offset(mem::size_of::<Rsdt>() as isize)
                        .offset(self.cur * 4);
                    let address = (ptr as *const u32).read_unaligned();
                    (address as *const sdt::SdtHeader).as_ref().unwrap()
                },
                AcpiTables::Extended(ref xsdt) => unsafe {
                    let ptr = (*xsdt as *const _ as *const u8)
                        .offset(mem::size_of::<Xsdt>() as isize)
                        .offset(self.cur * 8);
                    let address = (ptr as *const u64).read_unaligned();
                    (address as *const sdt::SdtHeader).as_ref().unwrap()
                },
            };

            self.cur += 1;

            unsafe {
                match header.signature {
                    madt::Madt::SIGNATURE => Some(TableKind::Madt(
                        (header as *const _ as *const madt::Madt).as_ref().unwrap(),
                    )),
                    _ => Some(TableKind::Unknown(&header)),
                }
            }
        }
    }
}
