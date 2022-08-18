/*
 * Here we specify the GDT to use.
 *
 * See sections 3.4 and 5.2 in Intel manual 3A.
 */
.section .rodata.gdt, "a", @progbits
.balign 8
gdt:
    /* null entry */
    .word   gdt_end - gdt - 1
    .long   gdt
    .word   0

    /* 64-bit code descriptor */
    .short  0xffff      /* limit 15:0 */
    .short  0           /* base 31:16 */
    .byte   0           /* 23:16 */
    .byte   0b10011010  /* P(1) DPL(00) S(1) 1 C(0) R(1) A(0) */
    .byte   0b00101111  /* G(0) D(0) L(1) AVL(0) seg limit 19:16 */
    .byte   0           /* base 31:24 */

    /* data descriptor */
    .short  0xffff      /* limit 15:0 */
    .short  0           /* base 31:16 */
    .byte   0           /* 23:16 */
    .byte   0b10010010  /* P(1) DPL(00) S(1) 0 E(0) W(1) A(0) */
    .byte   0b11001111  /* G(1) B(1) 0 AVL(0) seg limit 19:16 */
    .byte   0           /* base 31:24 */
gdt_end:
