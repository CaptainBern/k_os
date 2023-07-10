//! IOAPIC registers.
//!
//! These are very similar to the x(2)APIC registers, but not entirely the same.

pub const IO_APIC_REG_SEL: u32 = 0x00;
pub const IO_APIC_REG_WIN: u32 = 0x10;

pub const IO_APIC_ID_REG: u32 = 0x00;
pub const IO_APIC_VERSION_REG: u32 = 0x01;
pub const IO_APIC_ARB_ID_REG: u32 = 0x02;

pub const IO_APIC_RED_TBL_0: u32 = 0x10;
pub const IO_APIC_RED_TBL_1: u32 = 0x12;
pub const IO_APIC_RED_TBL_2: u32 = 0x14;
pub const IO_APIC_RED_TBL_3: u32 = 0x16;
pub const IO_APIC_RED_TBL_4: u32 = 0x18;
pub const IO_APIC_RED_TBL_5: u32 = 0x1a;
pub const IO_APIC_RED_TBL_6: u32 = 0x1c;
pub const IO_APIC_RED_TBL_7: u32 = 0x1e;
pub const IO_APIC_RED_TBL_8: u32 = 0x20;
pub const IO_APIC_RED_TBL_9: u32 = 0x22;
pub const IO_APIC_RED_TBL_10: u32 = 0x24;
pub const IO_APIC_RED_TBL_11: u32 = 0x26;
pub const IO_APIC_RED_TBL_12: u32 = 0x28;
pub const IO_APIC_RED_TBL_13: u32 = 0x2a;
pub const IO_APIC_RED_TBL_14: u32 = 0x2c;
pub const IO_APIC_RED_TBL_15: u32 = 0x2e;
pub const IO_APIC_RED_TBL_16: u32 = 0x30;
pub const IO_APIC_RED_TBL_17: u32 = 0x32;
pub const IO_APIC_RED_TBL_18: u32 = 0x34;
pub const IO_APIC_RED_TBL_19: u32 = 0x36;
pub const IO_APIC_RED_TBL_20: u32 = 0x38;
pub const IO_APIC_RED_TBL_21: u32 = 0x3a;
pub const IO_APIC_RED_TBL_22: u32 = 0x3c;
pub const IO_APIC_RED_TBL_23: u32 = 0x3e;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct IoApicId {
    bits: u32,
}

impl IoApicId {
    pub const fn new(id: u8) -> Self {
        Self {
            bits: ((id & 0xf) as u32) << 24,
        }
    }

    pub const fn ioapic_id(&self) -> u8 {
        ((self.bits >> 24) & 0xf) as u8
    }

    pub fn set_ioapic_id(&mut self, id: u8) {
        self.bits &= !(0xf << 24);
        self.bits |= ((id & 0xf) as u32) << 24;
    }

    pub const unsafe fn from_bits_unchecked(bits: u32) -> Self {
        Self { bits }
    }

    pub const fn bits(&self) -> u32 {
        self.bits
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Version {
    bits: u32,
}

impl Version {
    pub const fn version(&self) -> u8 {
        (self.bits & 0xff) as u8
    }

    pub const fn max_redir_entry(&self) -> u8 {
        ((self.bits >> 16) & 0xff) as u8
    }

    pub const unsafe fn from_bits_unchecked(bits: u32) -> Self {
        Self { bits }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Arbitration {
    bits: u32,
}

impl Arbitration {
    pub const fn ioapic_id(&self) -> u8 {
        ((self.bits >> 24) & 0xff) as u8
    }

    pub const unsafe fn from_bits_unchecked(bits: u32) -> Self {
        Self { bits }
    }

    pub const fn bits(&self) -> u32 {
        self.bits
    }
}

/// Interrupt delivery mode.
///
/// Specifies the type of interrupt to be sent to the processor. While these
/// are very similar to the ones specified in [crate::apic::registers::DeliveryMode],
/// they are not exactly the same or completely interchangeable. Therefore
/// we're redefining them here.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum DeliveryMode {
    /// Deliver the signal on the INTR signal of all processor cores listed
    /// in the destination.
    Fixed = 0b000,

    /// Deliver the signal on the INTR signal of the processor core that is
    /// executing at the lowest priority among all the processors listed in
    /// the specified destination. Trigger mode for 'lowest priority'.
    /// Delivery mode can be edge or level.
    LowestPriority = 0b001,

    /// System Management Interrupt. A delivery mode equal to SMI requires
    /// an edge trigger mode. The vector information is ignored but must be
    /// set to all zeroes.
    SMI = 0b010,

    /// Reserved.
    _Reserved0 = 0b011,

    /// Delivers the signal on the NMI signal of all processor cores listed
    /// in the destination. Vector information is ignored. NMI is treated
    /// as an edge-triggered interrupt, even when programmed as a level-
    /// triggered interrupt. For proper operation, this redirection table
    /// entry must be programmed to 'edge' triggered interrupt.
    NMI = 0b100,

    /// Deliver the signal to all processor cores listed in the destination by
    /// asserting the INIT signal. All addressed local APICs will assume their
    /// INIT state. INIT is always treated as an edge-triggered interrupt, even
    /// when programmed otherwise. For proper operation and future
    /// compatibility, this redirection table entry must be programmed to be
    /// 'edge' triggered.
    INIT = 0b101,

    /// Reserved.
    _Reserved1 = 0b110,

    /// Delivers the signal to the INTR signal of all processor cores listed in
    /// the destination as an interrupt that originated in an externally
    /// connected (8259A-compatible) interrupt controller. The INTA cycle that
    /// corresponds to this ExtINT delivery is routed to the external
    /// controller that is expected to supply the vector. A Delivery Mode of
    /// ExtINT requires an edge trigger mode.
    ExtINT = 0b111,
}

/// Interrupt Destination Mode.
///
/// This determines the interpretation of the destination field of the
/// [RedirectionTableEntry].
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum DestinationMode {
    /// Selects physical destination mode.
    ///
    /// The destination field must be an APIC ID.
    Physical = 0,

    /// Selects logical destination mode.
    ///
    /// Destinations are identified by matching on the logical destination
    /// under the control of the Destination Format Register and the Logical
    /// Destination Register in each Local APIC.
    Logical = 1,
}

/// Indicates the interrupt delivery status.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum DeliveryStatus {
    /// There is currently no activity for this interrupt source, or the
    /// previous interrupt from this source was delivered to the processor core
    /// and accepted.
    Idle = 0,

    /// Indicates that an interrupt from this source has been delivered to the
    /// processor core but has not yet been accepted.
    SendPending = 1,
}

/// Specifies the polarity of the interrupt pin.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptPinPolarity {
    HighActive = 0,
    LowActive = 1,
}

/// Indicates the type of signal on the interrupt pin that triggers an interrupt.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TriggerMode {
    Edge = 0,
    Level = 1,
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct RedirectionTableEntryLow {
    bits: u32,
}

impl RedirectionTableEntryLow {
    pub const fn new(
        vector: u8,
        delivery_mode: DeliveryMode,
        destination_mode: DestinationMode,
        int_pin_polarity: InterruptPinPolarity,
        trigger_mode: TriggerMode,
        masked: bool,
    ) -> Self {
        let mut bits = vector as u32;
        bits |= (delivery_mode as u32) << 8;
        bits |= (destination_mode as u32) << 11;
        bits |= (int_pin_polarity as u32) << 13;
        bits |= (trigger_mode as u32) << 15;
        bits |= (masked as u32) << 16;

        Self { bits }
    }

    pub const fn vector(&self) -> u8 {
        (self.bits & 0xff) as u8
    }

    pub fn set_vector(&mut self, vector: u8) {
        self.bits &= !0xff;
        self.bits |= vector as u32;
    }

    pub const fn delivery_mode(&self) -> DeliveryMode {
        match (self.bits >> 8) & 0b111 {
            0b000 => DeliveryMode::Fixed,
            0b001 => DeliveryMode::LowestPriority,
            0b010 => DeliveryMode::SMI,
            0b011 => DeliveryMode::_Reserved0,
            0b100 => DeliveryMode::NMI,
            0b101 => DeliveryMode::INIT,
            0b110 => DeliveryMode::_Reserved1,
            0b111 => DeliveryMode::ExtINT,
            _ => unreachable!(),
        }
    }

    pub fn set_delivery_mode(&mut self, mode: DeliveryMode) {
        self.bits &= !(0b111 << 8);
        self.bits |= (mode as u32) << 8;
    }

    pub const fn destination_mode(&self) -> DestinationMode {
        match (self.bits >> 11) & 1 {
            0 => DestinationMode::Physical,
            1 => DestinationMode::Logical,
            _ => unreachable!(),
        }
    }

    pub fn set_destination_mode(&mut self, mode: DestinationMode) {
        self.bits &= !(1 << 11);
        self.bits |= (mode as u32) << 11;
    }

    pub const fn delivery_status(&self) -> DeliveryStatus {
        match (self.bits >> 12) & 1 {
            0 => DeliveryStatus::Idle,
            1 => DeliveryStatus::SendPending,
            _ => unreachable!(),
        }
    }

    pub const fn int_pin_polarity(&self) -> InterruptPinPolarity {
        match (self.bits >> 13) & 1 {
            0 => InterruptPinPolarity::HighActive,
            1 => InterruptPinPolarity::LowActive,
            _ => unreachable!(),
        }
    }

    pub fn set_int_pin_polarity(&mut self, polarity: InterruptPinPolarity) {
        self.bits &= !(1 << 13);
        self.bits |= (polarity as u32) << 13;
    }

    /// Used for level triggered interrupts.
    ///
    /// This bit is set to 1 when local APIC(s) accept the level interrupt sent
    /// by the IOAPIC. The remote IRR bit is set to 0 when an EOI message with
    /// a matching interrupt vector is received from a local APIC.
    pub const fn remote_irr(&self) -> bool {
        ((self.bits >> 14) & 1) == 1
    }

    pub const fn trigger_mode(&self) -> TriggerMode {
        match (self.bits >> 15) & 1 {
            0 => TriggerMode::Edge,
            1 => TriggerMode::Level,
            _ => unreachable!(),
        }
    }

    pub fn set_trigger_mode(&mut self, mode: TriggerMode) {
        self.bits &= !(1 << 15);
        self.bits |= (mode as u32) << 15;
    }

    pub const fn masked(&self) -> bool {
        ((self.bits >> 16) & 1) == 1
    }

    pub fn set_masked(&mut self, masked: bool) {
        self.bits &= !(1 << 16);
        self.bits |= (masked as u32) << 16;
    }

    pub const unsafe fn from_bits_unchecked(bits: u32) -> Self {
        Self { bits }
    }

    pub const fn bits(&self) -> u32 {
        self.bits
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum LogicalDestination {
    ApicID(u8),
    Set(u8),
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct RedirectionTableEntryHigh {
    bits: u32,
}

impl RedirectionTableEntryHigh {
    pub const fn new(destination: LogicalDestination) -> Self {
        let bits = match destination {
            LogicalDestination::ApicID(id) => ((id & 0xf) as u32) << 24,
            LogicalDestination::Set(id) => ((id & 0xff) as u32) << 24,
        };

        Self { bits }
    }

    pub const fn logical_destination(&self, mode: DestinationMode) -> LogicalDestination {
        match mode {
            DestinationMode::Physical => {
                LogicalDestination::ApicID(((self.bits >> 24) & 0xf) as u8)
            }
            DestinationMode::Logical => LogicalDestination::Set(((self.bits >> 24) & 0xff) as u8),
        }
    }

    pub fn set_logical_destination(&mut self, destination: LogicalDestination) {
        self.bits &= !(0xff << 24); // clear 63:59 either way.

        self.bits |= match destination {
            LogicalDestination::ApicID(id) => ((id & 0xf) as u32) << 24,
            LogicalDestination::Set(id) => ((id & 0xff) as u32) << 24,
        };
    }

    pub const unsafe fn from_bits_unchecked(bits: u32) -> Self {
        Self { bits }
    }

    pub const fn bits(&self) -> u32 {
        self.bits
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RedirectionTableEntry {
    pub high: RedirectionTableEntryHigh,
    pub low: RedirectionTableEntryLow,
}

impl RedirectionTableEntry {
    pub const unsafe fn from_bits_unchecked(low: u32, high: u32) -> Self {
        Self {
            high: RedirectionTableEntryHigh::from_bits_unchecked(high),
            low: RedirectionTableEntryLow::from_bits_unchecked(low),
        }
    }
}
