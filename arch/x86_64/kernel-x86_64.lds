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
PERCPU_SIZE = 1 * 1024 * 1024;

/* Maximum number of supported CPUs. Lowering this value will save some memory. */
MAX_CPUS = 16;

/*
 * .percpu is linked at 0. If they happen to overflow beyond the physical offset, the linker will complain.
 */
ASSERT(PERCPU_SIZE <= KERNEL_PHYS_START, "PERCPU_SIZE may not exceed KERNEL_PHYS_START!");

/*
 * The KERNEL_SIZE may not exceed 1G. It's just for convenience since that way we can map everything
 * using a single PD.
 */
ASSERT(KERNEL_SIZE <= (1024 * 1024 * 1024), "KERNEL_SIZE may not exceed 1G!");

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

    .text.boot ALIGN(4K) : {
        *(.text.boot)
    }

    .text.boot16 ALIGN(4K) : {
        _boot16 = .;
        KEEP(*(.text.boot16))
        _eboot16 = .;
    }

    .bss.boot16 (NOLOAD) : {
        . += ALIGN(4K) + 0x1000;
    }

    /*
     * Start of the actual kernel.
     */
    . = . + VIRT_OFFSET;
    .text ALIGN(4K) : AT(ADDR(.text) - VIRT_OFFSET) {
        _text = .;
        *(.text)
        *(.text.*)
        _etext = .;
    }

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - VIRT_OFFSET) {
        _rodata = .;
        *(.rodata)
        *(.rodata.*)
        _erodata = .;
    }

    .data ALIGN(2M) : AT(ADDR(.data) - VIRT_OFFSET) {
        _data = .;
        *(.data)
        *(.data.*)
        _edata = .;
    }

    .bss (NOLOAD) : AT(ADDR(.bss) - VIRT_OFFSET) {
        _bss = .;
        *(.bss)
        *(.bss.*)
        _ebss = .;
    }

    _percpu_load = ALIGN(4K);
    .percpu 0 : AT(_percpu_load - VIRT_OFFSET) {
        *(.percpu)
        *(.percpu.*)
    }
    _epercpu_load = _percpu_load + SIZEOF(.percpu);
    . = _epercpu_load;

    /* Make sure we don't overlap with `.header` and `.text.boot(16)`. */
    ASSERT(SIZEOF(.percpu) <= KERNEL_PHYS_START, "CPU local too big!")

    _end = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
    }
}

/* Make sure the final size is not greater than the window we'll occupy. */
ASSERT((_end - KERNEL_START <= KERNEL_SIZE), "Kernel size is greater than KERNEL_SIZE!");