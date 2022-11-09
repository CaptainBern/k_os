OUTPUT_FORMAT("elf64-x86-64")
ENTRY(_start)

/* The offset of the kernel load addresss. */
KERNEL_LOAD_OFFSET = 0x100000;

/* The virtual offset of the kernel. */
KERNEL_OFFSET = 0xffffffff80000000;

KERNEL_STACK_SIZE = 0x4000;

/*PHDRS {
    phys PT_LOAD FILEHDR PHDRS ;
    boot PT_LOAD ;
    virt PT_LOAD ;

    text PT_LOAD;
    data PT_LOAD;
}*/

SECTIONS
{
    . = KERNEL_LOAD_OFFSET + SIZEOF_HEADERS;

    .phys : {
        KEEP(*(.phys.header))
        *(.phys.text)
        *(.phys.data)
    }

    .phys.bss 4K (NOLOAD) : {
        _phys_bss = .;
        *(.phys.bss)
        . = ALIGN(16);
        _ephys_bss = .;
    }

    . = ALIGN(4K) + KERNEL_OFFSET;
    
    .text : AT(ADDR(.text) - KERNEL_OFFSET) {
        _text = .;
        *(.text)
        *(.text.*)
        _etext = .;
    }

    .rodata : AT(ADDR(.rodata) - KERNEL_OFFSET) {
        _rodata = .;
        *(.rodata)
        *(.rodata.*)
        _erodata = .;
    }

    .data : AT(ADDR(.data) - KERNEL_OFFSET) {
        _data = .;
        *(.data)
        *(.data.*)
        _edata = .;
    }

    .bss : AT(ADDR(.bss) - KERNEL_OFFSET) {
        _bss = .;
        *(.bss)
        *(.bss.*)
        _ebss = .;
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
    }
}