use bitflags::bitflags;

use crate::{address::GenericAddress, sdt::SdtHeader, AcpiTable};

/// Fixed ACPI description table.
///
/// See ACPI v6.4 section 5.2.9
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Fadt {
    pub header: SdtHeader,
    pub firmware_ctrl: u32,
    pub dsdt: u32,
    pub _reserved0: u8,
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
    pub pm1_evt_len: u8,
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
    pub iapc_boot_arch: IaPCBootArchFlags,
    pub _reserved1: u8,
    pub flags: FixedFeatureFlags,
    pub reset_reg: GenericAddress,
    pub reset_value: u8,
    pub arm_boot_arch: ArmBootArchFlags,
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
    pub x_gpe1_bk: GenericAddress,
    pub sleep_control_reg: GenericAddress,
    pub sleep_status_reg: GenericAddress,
    pub hypervisor_identity_reg: u64,
}

impl AcpiTable for Fadt {
    const SIGNATURE: [u8; 4] = *b"FACP";
}

bitflags! {
    /// IA-PC boot architecture flags.
    ///
    /// See ACPI v6.4 section 5.2.9.3
    #[derive(Debug, Clone, Copy)]
    pub struct IaPCBootArchFlags: u16 {
        const LEGACY_DEVICES = 1 << 0;
        const MC8042 = 1 << 1;
        const VGA_NOT_PRESENT = 1 << 2;
        const MSI_NOT_SUPPORTED = 1 << 3;
        const PCIE_ASPM_CONTROLS = 1 << 4;
        const CMOS_RTC_NOT_PRESENT = 1 << 5;
    }
}

bitflags! {
    /// ARM architecture boot flags.
    ///
    /// See ACPI v6.4 section 5.2.9.4
    #[derive(Debug, Clone, Copy)]
    pub struct ArmBootArchFlags: u16 {
        const PSCI_COMPLIANT = 1 << 0;
        const PSCI_USE_HVC = 1 << 1;
    }
}

bitflags! {
    /// Fixed ACPI descirption table feature flags.
    ///
    /// See ACPI v6.4 table 5.10
    #[derive(Debug, Clone, Copy)]
    pub struct FixedFeatureFlags: u32 {
        const WBINVD = 1 << 0;
        const WBIND_FLUSH = 1 << 1;
        const PROC_C1 = 1 << 2;
        const P_LVL2_UP = 1 << 3;
        const PWR_BUTTON = 1 << 4;
        const SLP_BUTTON = 1 << 5;
        const FIX_RTC = 1 << 6;
        const RTC_s4 = 1 << 7;
        const TMR_VAL_EXT = 1 << 8;
        const DCK_CAP = 1 << 9;
        const RESET_REG_SUP = 1 << 10;
        const SEALED_CASE = 1 << 11;
        const HEADLESS = 1 << 12;
        const CPU_SW_SLP = 1 << 13;
        const PCI_EXP_WK = 1 << 14;
        const USE_PLATFORM_CLOCK = 1 << 15;
        const S4_RTC_STS_VALID = 1 << 16;
        const REMOTE_POWER_ON_CAPABLE = 1 << 17;
        const FORCE_APIC_CLUSTER_MODEL = 1 << 18;
        const FORCE_APIC_PHYSICAL_DESTINATION_MODE = 1 << 19;
        const HW_REDUCED_ACPI = 1 << 20;
        const LOW_POWER_S0_IDLE_CAPABLE = 1 << 21;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PmProfile {
    Unspecified = 0,
    Desktop,
    Mobile,
    Workstation,
    EnterpriseServer,
    SOHOServer,
    AppliancePC,
    PerformanceServer,
    Tablet,
}
