use x86::io::outb;

const PIC1: u16 = 0x20;
const PIC2: u16 = 0xA0;

const ICW1_ICW4: u8 = 0x01;
const ICW1_INIT: u8 = 0x10;

/// Disable the PIC.
/// Note: this needs to happen *after* remapping of the PICs!
pub unsafe fn disable() {
    outb(PIC1 + 1, 0xff);
    outb(PIC2 + 1, 0xff);
}

/// Initialise and remap the PICs.
pub unsafe fn remap(offset1: u8, offset2: u8) {
    // Start init sequence.
    outb(PIC1, ICW1_ICW4 | ICW1_INIT);
    outb(PIC2, ICW1_ICW4 | ICW1_INIT);

    // ICW1
    outb(PIC1 + 1, offset1);
    outb(PIC2 + 1, offset2);

    // ICW2
    outb(PIC1 + 1, 0x4);
    outb(PIC2 + 1, 0x2);

    // ICW3
    outb(PIC1 + 1, 0x1);
    outb(PIC2 + 1, 0x1);

    // ICW4
    outb(PIC1 + 1, 0x0);
    outb(PIC2 + 1, 0x0);
}
