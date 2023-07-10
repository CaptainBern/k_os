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

use sdt::SdtHeader;

pub mod address;
pub mod fadt;
pub mod madt;
pub mod sdt;

pub type Result<T> = result::Result<T, AcpiError>;

#[derive(Debug)]
pub enum AcpiError {
    InvalidHeader,
    UnsupportedRevision,
    ChecksumFailed,
}

/// An ACPI table type.
pub trait AcpiTable {
    const SIGNATURE: [u8; 4];
}

/// ACPI version.
///
/// If present, the XSDT should be used over the RSDT.
#[derive(Debug)]
pub enum Version<'a> {
    /// Root System Description Table.
    ///
    /// The array of entries are physical pointers to various other system description
    /// tables.
    /// See ACPI v6.4 section 5.2.7
    Root(&'a SdtHeader),

    /// Extended System Description Table.
    ///
    /// Functionaly identical to [Rsdt], but instead using 64-bit pointers.
    /// See ACPI v6.4 section 5.2.8
    Extended(&'a SdtHeader),
}

impl<'a> Version<'a> {
    pub unsafe fn from_address(address: usize) -> Result<Self> {
        let header = (address as *const SdtHeader)
            .as_ref()
            .ok_or(AcpiError::InvalidHeader)?;

        // Make sure the header is valid.
        header.validate()?;

        match header.signature() {
            Ok("RSDT") => Ok(Version::Root(header)),
            Ok("XSDT") => Ok(Version::Extended(header)),
            _ => Err(AcpiError::InvalidHeader),
        }
    }

    /// Return the raw header.
    pub fn header(&self) -> &'a SdtHeader {
        match self {
            Version::Root(header) => header,
            Version::Extended(header) => header,
        }
    }
}

/// Enum wrapper for ACPI tables.
#[derive(Debug)]
pub struct AcpiTables<'a> {
    version: Version<'a>,
    offset: usize,
}

impl<'a> AcpiTables<'a> {
    /// Initialise the ACPI tables for a given RSDT address.
    ///
    /// # Safety
    /// The given address should point to a valid RSDT. The header is validated
    /// (which means its signature and checksum are checked) before it is
    /// accepted. However, a given address that points to a _valid_ RSDT
    /// containing bogus data can still cause unexpected behaviour.
    pub unsafe fn from_address(addr: usize, offset: usize) -> Result<Self> {
        let version = Version::from_address(addr + offset)?;
        Ok(Self { version, offset })
    }

    /// Compute the number of entries in the table.
    ///
    /// An RSDT contains 32-bit pointers, while an XSDT contains 64-bit pointers.
    pub fn len(&self) -> usize {
        let header = self.version.header();
        let size = match self.version {
            Version::Root(_) => 4,
            Version::Extended(_) => 8,
        };

        (header.length as usize)
            .checked_sub(mem::size_of::<sdt::SdtHeader>())
            .unwrap_or(0)
            / size
    }

    /// Return an iterator over the entries in the table.
    pub fn iter(&self) -> Entries {
        Entries {
            tables: self,
            len: self.len(),
            cur: 0,
        }
    }

    /// Compute the size of all the ACPI tables.
    pub fn size(&self) -> usize {
        self.version.header().length as usize
            + self
                .iter()
                .map(|table| table.header().length as usize)
                .sum::<usize>()
    }
}

#[derive(Debug)]
pub enum TableKind<'a> {
    Fadt(&'a fadt::Fadt),
    Madt(&'a madt::Madt),
    Unknown(&'a sdt::SdtHeader),
}

impl<'a> TableKind<'a> {
    #[inline]
    pub fn header(&self) -> &sdt::SdtHeader {
        match self {
            TableKind::Fadt(fadt) => &fadt.header,
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
            let header_address = match self.tables.version {
                Version::Root(ref rsdt) => unsafe {
                    let ptr = (*rsdt as *const _ as *const u8)
                        .offset(mem::size_of::<SdtHeader>() as isize)
                        .offset(self.cur * 4);
                    (ptr as *const u32).read_unaligned() as usize
                },
                Version::Extended(ref xsdt) => unsafe {
                    let ptr = (*xsdt as *const _ as *const u8)
                        .offset(mem::size_of::<SdtHeader>() as isize)
                        .offset(self.cur * 8);
                    (ptr as *const u64).read_unaligned() as usize
                },
            };

            let header = unsafe {
                ((header_address + self.tables.offset) as *const SdtHeader)
                    .as_ref()
                    .unwrap()
            };

            self.cur += 1;

            unsafe {
                if header.validate().is_err() {
                    None
                } else {
                    match header.signature {
                        fadt::Fadt::SIGNATURE => Some(TableKind::Fadt(
                            (header as *const _ as *const fadt::Fadt).as_ref().unwrap(),
                        )),
                        madt::Madt::SIGNATURE => Some(TableKind::Madt(
                            (header as *const _ as *const madt::Madt).as_ref().unwrap(),
                        )),
                        _ => Some(TableKind::Unknown(&header)),
                    }
                }
            }
        }
    }
}
