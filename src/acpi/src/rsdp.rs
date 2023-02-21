/// Root System Description Pointer.
///
/// The revision number indicates the size. (2 == RsdpV2).
/// See ACPI v6.4 section 5.2.5.3
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Rsdp {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oemid: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
}

impl Rsdp {
    #[allow(dead_code)]
    pub const SIGNATURE: [u8; 8] = *b"RSD PTR ";
}

/// Root System Description Pointer for version 2.
///
/// See ACPI v6.4 section 5.2.5.3
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct RsdpV2 {
    pub rsdp: Rsdp,
    pub length: u32,
    pub xsdt_address: u64,
    pub extended_checksum: u8,
    pub _reserved: [u8; 3],
}
