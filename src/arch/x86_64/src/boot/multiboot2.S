MB2_MAGIC   = 0xe85250d6
MB2_ARCH    = 0
MB2_HDR_LEN = (__mb2_header_end - __mb2_header_start)
MB2_CHKSM   = -(MB2_MAGIC + MB2_HDR_LEN)

.section .phys.mbh, "a", @progbits
.balign 8
__mb2_header_start:
    .long MB2_MAGIC
    .long MB2_ARCH
    .long MB2_HDR_LEN
    .long MB2_CHKSM
__mb2_end_tag_start:
    .word 0
    .word 0
    .long (__mb2_end_tag_end - __mb2_end_tag_start)
__mb2_end_tag_end:
__mb2_header_end:
