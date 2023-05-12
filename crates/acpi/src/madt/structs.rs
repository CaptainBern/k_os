use bitflags::bitflags;

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
pub struct ProcessorLocalApicStructure {
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
    pub io_apic_id: u8,
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
    pub flags: MpsIntiFlags,
}

bitflags! {
    /// MPS INTI Flags.
    ///
    /// See ACPI v6.4 section 5.2.12.5 (table 5.25)
    #[derive(Debug, Clone, Copy)]
    pub struct MpsIntiFlags: u16 {
        const POLARITY_CONFORMS = 0 << 0;
        const POLARITY_ACTIVE_HIGH = 1 << 0;
        const POLARITY_RESERVED = 2 << 0;
        const POLARITY_ACTIVE_LOW = 3 << 0;

        const TRIGGER_MODE_CONFORMS = 0 << 2;
        const TRIGGER_MODE_EDGE = 1 << 2;
        const TRIGGER_MODE_RESERVED = 2 << 2;
        const TRIGGER_MODE_LEVEL = 3 << 2;
    }
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

/// Local APIC NMI Structure.
///
/// See ACPI v6.4 section 5.2.12.7
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct LocalApicNmiStructure {
    pub header: ApicStructureHeader,
    pub _reserved: [u8; 2],
    pub local_apic_address: u64,
}

/// Local APIC Override Structure.
///
/// See ACPI v6.4 section 5.2.12.8
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct LocalApicAdressOverrideStructure {
    pub header: ApicStructureHeader,
    pub _reserved: [u8; 2],
    pub local_apic_address: u64,
}

/// IO SAPIC Structure.
///
/// See ACPI v6.4 section 5.2.12.9
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IoSapicStructure {
    pub header: ApicStructureHeader,
    pub io_apic_id: u8,
    pub _reserved: u8,
    pub global_system_interrupt_base: u32,
    pub io_sapic_address: u64,
}

/// Local SAPIC Structure.
///
/// See ACPI v6.4 section 5.2.12.10
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct LocalSapicStructure {
    pub header: ApicStructureHeader,
    pub apic_processor_id: u8,
    pub local_sapic_id: u8,
    pub local_sapic_eid: u8,
    pub _reserved: [u8; 3],
    pub flags: LocalApicFlags,
    pub acpi_processor_uid_value: u32,
    // Special case: null terminated string
    pub acpi_processor_uid_string: [u8; 1],
}

/// Platform Interrupt Source Structure.
///
/// See ACPI v6.4 section 5.2.12.11
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct PlatformInterruptSourceStructure {
    pub header: ApicStructureHeader,
    pub flags: MpsIntiFlags,
    pub interrupt_type: InterruptType,
    pub processor_id: u8,
    pub processor_eid: u8,
    pub io_sapic_vector: u8,
    pub global_system_interrupt: u32,
    pub platform_interrupt_source_flags: PlatformInterruptSourceFlags,
}

bitflags! {
    /// Platform Interrupt Source flags.
    ///
    /// See ACPI v6.4 section 5.2.12.11 (table 5.33)
    #[derive(Debug, Clone, Copy)]
    pub struct PlatformInterruptSourceFlags: u32 {
        const CPEI_PROCESSOR_OVERRIDE = 1 << 0;
    }
}

/// Platform Interrupt Source interrupt type.
///
/// See ACPI v6.4 section 5.2.12.11 (table 5.32)
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptType {
    Pmi = 0,
    Init = 1,
    CorrectedPlatformErrorInterrupt = 3,
}

/// Processor Local x2APIC Structure.
///
/// See ACPI v6.4 section 5.2.12.12
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ProcessorLocalX2ApicStructure {
    pub header: ApicStructureHeader,
    pub _reserved: [u8; 2],
    pub x2apic_id: u32,
    pub flags: LocalApicFlags,
    pub acpi_processor_uid: u32,
}

/// Local x2APIC NMI Structure.
///
/// See ACPI v6.4 section 5.2.12.13
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct LocalX2ApicNmiStructure {
    pub header: ApicStructureHeader,
    pub flags: MpsIntiFlags,
    pub acpi_processor_uid: u32,
    pub local_x2apic_lint_n: u8,
    pub _reserved: [u8; 3],
}

/// GIC CPU Interface Structure.
///
/// See ACPI v6.4 section 5.2.12.14
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GiccStructure {
    pub header: ApicStructureHeader,
    pub _reserved1: [u8; 2],
    pub cpu_interface_number: u32,
    pub acpi_processor_uid: u32,
    pub flags: GiccFlags,
    pub parking_protocol_version: u32,
    pub performance_interrupt_gsiv: u32,
    pub parked_address: u64,
    pub physical_base_address: u64,
    pub gicv: u64,
    pub gich: u64,
    pub vgic_maintenance_interrupt: u32,
    pub gicr_base_address: u64,
    pub mpidr: u64,
    pub processor_power_efficiency_class: u8,
    pub _reserved2: [u8; 1],
    pub spe_overflow_interrupt: u16,
}

bitflags! {
    /// GICC CPU Interface Flags.
    ///
    /// See ACPI v6.4 section 5.2.12.14 (table 5.37)
    #[derive(Debug, Clone, Copy)]
    pub struct GiccFlags: u32 {
        const ENABLED = 1 << 0;
        const PERFORMANCE_INTERRUPT_MODE = 1 << 1;
        const VGIC_MAINTENANCE_INTERRUPT_MODE_FLAGS = 1 << 2;
    }
}

/// GIC Distributor Structure.
///
/// See ACPI v6.4 section 5.2.12.15
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GicdStructure {
    pub header: ApicStructureHeader,
    pub _reserved1: [u8; 2],
    pub gic_id: u32,
    pub physical_base_address: u64,
    pub system_vector_base: u32,
    pub gic_version: u8,
    pub _reserved2: [u8; 3],
}

/// GIC MSI Frame Structure.
///
/// See ACPI v6.4 section 5.2.12.16
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GicMsiFrameStructure {
    pub header: ApicStructureHeader,
    pub _reserved: [u8; 2],
    pub gic_msi_frame_id: u32,
    pub physical_base_address: u64,
    pub flags: GicMsiFrameFlags,
    pub spi_count: u16,
    pub spi_base: u16,
}

bitflags! {
    /// GIC MSI Frame Flags.
    ///
    /// See ACPI v6.4 section 5.2.12.16 (table 5.40)
    #[derive(Debug, Clone, Copy)]
    pub struct GicMsiFrameFlags: u32 {
        const SPI_COUNT_BASE_SELECT = 1;
    }
}

/// GIC Redistributor Structure.
///
/// See ACPI v6.4 section 5.2.12.17
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GicrStructure {
    pub header: ApicStructureHeader,
    pub _reserved: [u8; 2],
    pub discovery_range_base_address: u64,
    pub discovery_range_length: u32,
}

/// GIC Interrupt Translation Service Structure.
///
/// See ACPI v6.4 section 5.2.12.18
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GicItsStructure {
    pub header: ApicStructureHeader,
    pub _reserved1: [u8; 2],
    pub gic_its_id: u32,
    pub physical_base_address: u64,
    pub _reserved2: [u8; 4],
}

/// Multiprocessor Wakeup Structure.
///
/// See ACPI v6.4 section 5.2.12.19
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MultiProcessorWakeupStructure {
    pub header: ApicStructureHeader,
    pub mailbox_version: u16,
    pub _reserved: [u8; 4],
    pub mailbox_address: u64,
}

/// Multiprocessor Wakeup Mailbox Structure.
///
/// See ACPI v6.4 section 5.2.12.19 (table 5.44)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MultiProcessorWakeupMailbox {
    pub command: MultiProcessorWakeupMailboxCommand,
    pub _reserved: [u8; 2],
    pub apic_id: u32,
    pub wakeup_vector: u64,
    pub reserved_for_os: [u8; 2032],
    pub reserved_for_firmware: [u8; 2048],
}

/// Multiprocessor Wakeup Mailbox Command.
///
/// See ACPI v6.4 section 5.2.12.19 (table 5.44)
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum MultiProcessorWakeupMailboxCommand {
    Nop = 0,
    Wakeup = 1,
}
