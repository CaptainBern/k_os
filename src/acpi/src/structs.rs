//! This module contains the 'raw' (low-level) structs used by ACPI. They are a
//! direct implementation of the structures described in ACPI Spec v6.4.
//!
//! The top-level 'lib' (and other) modules provide a higher-level interface to
//! safely work with these structs.

use bitflags::bitflags;

use crate::AcpiTable;

macro_rules! signature {
    ($t:ty, $s:literal) => {
        impl AcpiTable for $t {
            const SIGNATURE: [u8; 4] = *$s;
        }
    };
}

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

/// Root System Description Table.
///
/// The array of entries are physical pointers to various other system description
/// tables.
/// See ACPI v6.4 section 5.2.7
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Rsdt {
    pub header: SdtHeader,
    pub entries: [u32; 1],
}
signature!(Rsdt, b"RSDT");

/// Extended System Description Table.
///
/// Functionaly identical to [Rsdt], but instead using 64-bit pointers.
/// See ACPI v6.4 section 5.2.8
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Xsdt {
    pub header: SdtHeader,
    pub entries: [u64; 1],
}
signature!(Xsdt, b"XSDT");

/// Fixed ACPI Description Table.
///
/// See ACPI v6.4 section 5.2.9
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Fadt {
    pub header: SdtHeader,
    pub firmware_ctrl: u32,
    pub dsdt: u32,
    pub _reserved: u8,
    pub preferred_pm_profile: u8,
    pub sci_int: u16,
    pub smi_cmd: u32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
    pub s4bios_req: u8,
    pub pstate_cnt: u8,
    pub pm1a_evt_blk: u32,
    pub pm1b_evt_blk: u32,
    pub pm1a_cnt_blk: u32,
    pub pm1b_cnt_blk: u32,
    pub pm2_cnt_blk: u32,
    pub pm_tmr_blk: u32,
    pub gpe0_blk: u32,
    pub gpe1_blk: u32,
    pub pm1_vt_len: u8,
    pub pm1_cnt_len: u8,
    pub pm2_cnt_len: u8,
    pub pm_tmr_len: u8,
    pub gpe0_blk_len: u8,
    pub gpe1_blk_len: u8,
    pub gpe1_base: u8,
    pub cst_cnt: u8,
    pub p_lvl2_lat: u16,
    pub p_lvl3_lat: u16,
    pub flush_size: u16,
    pub flush_stride: u16,
    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alrm: u8,
    pub mon_alrm: u8,
    pub century: u8,
    pub iapc_boot_arch: u16,
    pub _reserved2: u8,
    pub flags: u32,
    pub reset_reg: GenericAddress,
    pub reset_value: u8,
    pub arm_boot_arch: u16,
    pub fadt_minor_version: u8,
    pub x_firmware_ctrl: u64,
    pub x_dsdt: u64,
    pub x_pm1a_evt_blk: GenericAddress,
    pub x_pm1b_evt_blk: GenericAddress,
    pub x_pm1a_cnt_blk: GenericAddress,
    pub x_pm1b_cnt_blk: GenericAddress,
    pub x_pm2_cnt_blk: GenericAddress,
    pub x_pm_tmr_blk: GenericAddress,
    pub x_gpe0_blk: GenericAddress,
    pub x_gpe1_blk: GenericAddress,
    pub sleep_control_reg: GenericAddress,
    pub sleep_status_reg: GenericAddress,
    pub hypervisor_vendor_identity: u64,
}
signature!(Fadt, b"FACP");

/// Firmware ACPI Control Structure.
///
/// FACS is passed using the [Fadt].
/// See ACPI v6.4 section 5.2.10
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Facs {
    pub signature: [u8; 4],
    pub length: u32,
    pub hardware_signature: u32,
    pub firmware_waking_vector: u32,
    pub global_lock: u32,
    pub flags: FcsfFlags,
    pub x_firmware_waking_vector: u64,
    pub version: u8,
    pub _reserved: [u8; 3],
    pub ospm_flags: OspmFcsfFlags,
    pub _reserved2: [u8; 24],
}
signature!(Facs, b"FACS");

bitflags! {
    /// Firmware Control Structure Feature Flags.
    ///
    /// See ACPI v6.4 table 5.14
    #[derive(Debug, Clone, Copy)]
    pub struct FcsfFlags: u32 {
        const S4BIOS = 1 << 0;
        const BIT64_WAKE_SUPPORTED = 1 << 1;
    }

    /// OSPM Enabled Firmware Control Structure Feature Flags.
    ///
    /// See ACPI v6.4 table 5.15
    #[derive(Debug, Clone, Copy)]
    pub struct OspmFcsfFlags: u32 {
        const BIT64_WAKE = 1 << 0;
    }
}

/// Multiple APIC Description Table.
///
/// See ACPI v6.4 section 5.2.12
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Madt {
    pub header: SdtHeader,
    pub local_interrupt_controller_address: u32,
    pub flags: MaFlags,
    pub interrupt_controller_structures: [ApicStructureHeader; 1],
}
signature!(Madt, b"APIC");

bitflags! {
    /// Multiple APIC Flags.
    ///
    /// See ACPI v6.4 table 5.20
    #[derive(Debug, Clone, Copy)]
    pub struct MaFlags: u32 {
        const PCAT_COMPAT = 1;
    }
}

/// Interrupt Controller Structure Header.
///
/// Each Interrupt Controller Structure starts with two bytes.
/// The first byte declares the type of the structure, the second byte
/// declares the length of the structure.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ApicStructureHeader {
    pub entry_type: u8,
    pub length: u8,
}

/// Processor Local APIC Structure.
///
/// See ACPI v6.4 section 5.2.12.2
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct LocalApicStructure {
    pub header: ApicStructureHeader,
    pub acpi_processor_uid: u8,
    pub apic_id: u8,
    pub flags: LocalApicFlags,
}

bitflags! {
    /// Local APIC Flags.
    ///
    /// See ACPI v6.4 table 5.23
    #[derive(Debug, Clone, Copy)]
    pub struct LocalApicFlags: u32 {
        const ENABLED = 1 << 0;
        const ONLINE_CAPABALE = 1 << 1;
    }
}

/// I/O APIC Structure.
///
/// See ACPI v6.4 section 5.2.12.3
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IoApicStructure {
    pub header: ApicStructureHeader,
    pub io_apic_ID: u8,
    pub _reserved: u8,
    pub io_apic_address: u32,
    pub global_system_interrupt_base: u32,
}

/// Interrupt Source Override Structure.
///
/// See ACPI v6.4 section 5.2.12.5
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IntSourceOverrideStructure {
    pub header: ApicStructureHeader,
    pub bus: u8,
    pub source: u8,
    pub global_system_interrupt: u32,
    pub flags: u16,
}

/// Non-Maskable Interrupt Source Structure.
///
/// See ACPI v6.4 section 5.2.12.6
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct NmiSourceStructure {
    pub header: ApicStructureHeader,
    pub flags: u16,
    pub global_system_interrupt: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct LocalApicNmiStructure {
    pub header: ApicStructureHeader,
    pub _reserved: [u8; 2],
    pub local_apic_address: u64,
}
