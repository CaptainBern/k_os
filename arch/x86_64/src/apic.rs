#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Register {
    ApicID = 0x20,
    ApicVersion = 0x30,
    TaskPriority = 0x80,
    ArbitrationPriority = 0x90,
    ProcessorPriority = 0xa0,
    EndOfInterupt = 0xb0,
    RemoteRead = 0xc0,
    LogicalDestination = 0xd0,
    DestinationFormat = 0xe0,
    SpuriousInterruptVector = 0xf0,
    InService = 0x100,        /*0x100-0x170 */
    TriggerMode = 0x180,      /* 0x180-0x1f0 */
    InterruptRequest = 0x200, /* 0x200-0x270 */
    ErrorStatus = 0x280,
    InterruptCommandLow = 0x300,  /* bits 31:0 */
    InterruptCommandHigh = 0x310, /* bits 63:32 */
    TimerLocalVectorTableEntry = 0x320,
    ThermalLocalVectorTableEntry = 0x330,
    PerfCounterLocalVectorTableEntry = 0x340,
    LocalInterrupt0VectorTableEntry = 0x350,
    LocalInterrupt1VectorTableEntry = 0x360,
    ErrorVectorTableEntry = 0x370,
    TimerInitialCount = 0x380,
    TimerCurrentCount = 0x390,
    TimerDivideConfiguration = 0x3e0,
    ExtendedApicFeature = 0x400,
    ExtendedApicControl = 0x410,
    SpecificEndOfInterrupt = 0x420,
    InterruptEnable = 0x480,                   /* 0x480-0x4f0 */
    ExtendedInterruptLocalVectorTable = 0x500, /* 0x500-0x530 */
}

/// The local APIC, mapped into the 4K APIC register space.
#[derive(Debug)]
pub struct Apic<const OFFSET: usize>;

impl<const OFFSET: usize> Apic<OFFSET> {
    #[inline]
    pub unsafe fn raw_write(reg: Register, val: u32) {
        let raw = (OFFSET + reg as usize) as *mut u32;
        raw.write_volatile(val);
    }

    #[inline]
    pub unsafe fn raw_read(reg: Register) -> u32 {
        let raw = (OFFSET + reg as usize) as *const u32;
        raw.read_volatile()
    }
}
