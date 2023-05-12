OUTPUT_FORMAT("elf64-x86-64")
ENTRY(KERNEL_ENTRY)

/* 
 * The virtual offset of the kernel, mapped at physical address 0. The code in
 * `start.S` depends on this value being '0xffffffff80000000', any changes made
 * here should also be made in `start.S`.
 */
VIRT_OFFSET = 0xffffffff80000000;

/* The physical start address of the kernel. */
KERNEL_PHYS_START = 0x1000000;

/* The virtual start address of the kernel. */
KERNEL_START = KERNEL_PHYS_START + VIRT_OFFSET;

/* The size of the window the kernel image will occupy. */
KERNEL_SIZE = 512 * 1024 * 1024;

/* The maximum .percpu size. This value may not exceed KERNEL_PHYS_START. */
PERCPU_SIZE = 2 * 1024 * 1024;

/* Maximum number of supported CPUs. Lowering this value will save some memory. */
MAX_CPUS = 16;

/*
 * .percpu is linked at 0. If they happen to overflow beyond the physical offset, the linker will complain.
 */
ASSERT(PERCPU_SIZE < KERNEL_PHYS_START, "PERCPU_SIZE may not exceed KERNEL_PHYS_START!");

/*
 * The KERNEL_SIZE may not exceed 1G. It's just for convenience since that way we can map everything
 * using a single PD.
 */
ASSERT(KERNEL_SIZE < (1024 * 1024 * 1024), "KERNEL_SIZE may not exceed 1G!");

/*
 * The virtual window in which we store per-cpu data may not exceed 8G.
 */
ASSERT((MAX_CPUS * PERCPU_SIZE) <= (8 * 1024 * 1024 * 1024), 
    "percpu window may not exceed 8G! Lower MAX_CPUS or PERCPU_SIZE!");

SECTIONS
{
    . = KERNEL_PHYS_START;

    .header : {
        KEEP(*(.header))
        KEEP(*(.header.*))
    }

    . = ALIGN(4K);
    .text.boot : {
        *(.text.boot)
    }

    . = ALIGN(4K);
    .data.boot : {
        *(.data.boot)
    }

    . = ALIGN(4K);
    .bss.boot (NOLOAD) : {
        _bss_boot = .;
        *(.bss.boot)
        _ebss_boot = .;
    }

    /*
     * Start of the actual kernel.
     */
    . = ALIGN(4K) + VIRT_OFFSET;
    .text : AT(ADDR(.text) - VIRT_OFFSET) {
        _text = .;
        *(.text)
        *(.text.*)
        _etext = .;
    }

    . = ALIGN(4K);
    .rodata : AT(ADDR(.rodata) - VIRT_OFFSET) {
        _rodata = .;
        *(.rodata)
        *(.rodata.*)
        _erodata = .;
    }

    . = ALIGN(2M);
    .data : AT(ADDR(.data) - VIRT_OFFSET) {
        _data = .;
        *(.data)
        *(.data.*)
        _edata = .;
    }

    .bss : AT(ADDR(.bss) - VIRT_OFFSET) {
        _bss = .;
        *(.bss)
        *(.bss.*)
        _ebss = .;
    }

   /* . = ALIGN(4K);
    _percpu = .;
    .cpulocal 0 : AT(_percpu - VIRT_OFFSET) {
        *(.cpulocal)
        *(.cpulocal.*)
    }
    _epercpu = _cpulocal + SIZEOF(.percpu) + ALIGN(4K);

    /* Make sure we stay within our constraints. 
    ASSERT(SIZEOF(.percpu) <= PERCPU_SIZE, ".percpu too big!")

    . = _epercpu; */

    _end = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
    }
}

/* Make sure the final size is not greater than the window we'll occupy. */
ASSERT((_end - KERNEL_START <= KERNEL_SIZE), "Kernel size is greater than KERNEL_SIZE!");