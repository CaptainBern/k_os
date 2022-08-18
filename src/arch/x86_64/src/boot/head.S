.section .phys.text, "ax", @progbits
.code32

/* The entry-point of the kernel. */
.global start32
start32:
    cli
    cld

    /*
     * Check if we were booted through a multiboot2 compatible bootloader.
     * According to the spec, the bss section should be zeroed.
     */
    cmp     eax, 0x36d76289
    jne     hang

    /* Setup a temporary boot stack. */
    lea     esp, stack_top

    /* Store multiboot2 struct. */
    push    0
    push    ebx

    /* Check for CPUID and longmode support. */
    call    verify_cpu
    test    eax, eax
    jnz     hang

/*
 * Identity map the first 4GiB. Proper paging setup happens in
 * long mode.
 */
.Lsetup_paging:
    /* Setup PML4 paging */
    lea     edi, BOOT_PML4
    lea     eax, BOOT_PDPT
    or      eax, (1 << 1 | 1 << 0)          /* P + R/W */
    mov     [edi], eax

    /* Build the PDPT */ 
    lea     edi, BOOT_PDPT
    lea     eax, BOOT_PD
    or      eax, (1 << 1 | 1 << 0)          /* P + R/W */
    mov     ecx, 4
1:
    mov     [edi], eax
    add     edi, 8
    add     eax, 512 * 8
    loop    1b

    /* Finally build the PD */
    lea     edi, BOOT_PD
    mov     eax, (1 << 7 | 1 << 1 | 1 << 0) /* Huge + P + R/W */
    mov     ecx, 512 * 4
1:
    mov     [edi], eax
    add     edi, 8
    add     eax, 0x200000
    loop    1b

.Lenable_paging:
    /* Disable PG. */
    mov     eax, cr0
    and     eax, ~(1 << 31)
    mov     cr0, eax
    
    /* Enable PAE and PGE. */
    mov     eax, cr4
    or      eax, (1 << 7 | 1 << 5)
    mov     cr4, eax

    /* Load the PML4 table into cr3. */
    lea     eax, BOOT_PML4
    mov     cr3, eax

    /* Load GDT. */
    lgdt    [gdt]

    /* Enable long mode. */
    mov     ecx, 0xc0000080
    rdmsr
    or      eax, 1 << 8
    wrmsr

    /* Prepare for retf. */
    push    8
    lea     eax, start64
    push    eax

    /* Enable PG and PE */
    mov     eax, cr0
    or      eax, 1 << 31
    mov     cr0, eax

    /* Jump into long mode. */
    retf

/* Hang in case something goes wrong */
hang:
    hlt
    jmp hang

/* 
 * Check if the CPU is supported.
 * In short it checks for the following features:
 *  - cpuid being present
 *  - SSE being available
 *  - extended cpuid mode being present
 *  - long mode being supported
 */
verify_cpu:
    push    0
    popfd

    /*  Check if CPUID is present. */
    pushfd
    pushfd
    pop     eax
    mov     ebx, eax
    xor     eax, 1 << 21
    push    eax
    popfd
    pushfd
    pop     eax
    popfd
    cmp     ebx, eax
    jz      .Lno_longmode

    /* Check if SSE is available. */
    mov     eax, 0x1
    cpuid
    test    edx, 1 << 25
    jz      .Lno_longmode

    /* Check if extended mode is available. */
    mov     eax, 0x80000000
    cpuid
    cmp     eax, 0x80000001
    jb      .Lno_longmode

    /* Check if longmode is available. */
    mov     eax, 0x80000001
    cpuid
    test    edx, 1 << 29
    jz      .Lno_longmode

    xor     eax, eax
    ret

.Lno_longmode:
    mov     eax, 0x1
    ret

.code64
start64:
    /* Setup segment registers. */
    xor     eax, eax
    mov     ds, eax
    mov     es, eax
    mov     ss, eax
    mov     fs, eax
    mov     gs, eax

    /* Pop off multiboot parameter. */
    pop     rdi

    /* Call into rust. */
    call    boot
0:
    hlt
    jmp     0b

/*
 * Initial boot stack.
 */
.bss
.balign 4096
stack_bottom:
    .skip   0x4000
stack_top: