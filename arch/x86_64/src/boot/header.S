.section ".header", "a", @progbits
.align 8
__mb2_start:
.long   0xe85250d6
.long   0
.long   (__mb2_end - __mb2_start)
.long   -(0xe85250d6 + (__mb2_end - __mb2_start))

.word   0
.word   0
.long   0x8
__mb2_end: