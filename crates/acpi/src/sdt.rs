use core::{result, str};

use crate::{AcpiError, Result};

/// Generic System Description Table Header.
///
/// All system description tables begin with this header.
/// See ACPI v6.4 section 5.2.6
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct SdtHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oemid: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

impl SdtHeader {
    /// Validate the header.
    ///
    /// # Safety
    /// All the bytes in the header (+ `length - size_of::<SdtHeader>()`) are summed.
    /// When the result is 0, the header is considered valid.
    pub unsafe fn validate(&self) -> Result<()> {
        let ptr = self as *const SdtHeader as *const u8;
        let mut acc: u8 = 0;
        for i in 0..self.length {
            acc = acc.wrapping_add(ptr.offset(i as isize).read_unaligned());
        }

        if acc == 0 {
            Ok(())
        } else {
            Err(AcpiError::ChecksumFailed)
        }
    }

    pub fn signature(&self) -> result::Result<&str, str::Utf8Error> {
        str::from_utf8(&self.signature)
    }

    pub fn oemid(&self) -> result::Result<&str, str::Utf8Error> {
        str::from_utf8(&self.oemid)
    }

    pub fn oem_table_id(&self) -> result::Result<&str, str::Utf8Error> {
        str::from_utf8(&self.oem_table_id)
    }
}
