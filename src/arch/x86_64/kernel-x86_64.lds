OUTPUT_FORMAT("elf64-x86-64")
ENTRY(start32)

KERNEL_VADDR = 0xffffffff80000000;
KERNEL_PADDR = 0x100000;

KERNEL_OFFSET = KERNEL_VADDR - KERNEL_VADDR;

SECTIONS
{
    . = KERNEL_PADDR;

    .phys : {
        KEEP(*(.phys.mbh))
        *(.phys.text)
        *(.phys.data)
    }

    .text : {
        _text = .;
        *(.text)
        *(.text.*)
        _etext = .;
    }

    .rodata : {
        _rodata = .;
        *(.rodata)
        *(.rodata.*)
        _erodata = .;
    }

    .data : {
        _data = .;
        *(.data)
        *(.data.*)
        _edata = .;
    }

    .bss : {
        _bss = .;
        *(.bss)
        *(.bss.*)
        _ebss = .;
    }

    _end = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.debug_*)
    }
}