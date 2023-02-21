/// Generic Address Structure.
///
/// Expresses register addresses within tables defined by ACPI.
/// See ACPI v6.4 section 5.2.3.2
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GenericAddress {
    pub address_space_id: u8,
    pub register_bit_width: u8,
    pub register_bit_offset: u8,
    pub access_size: u8,
    pub address: u64,
}
