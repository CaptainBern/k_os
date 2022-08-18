# x86

Code:
    - check multiboot shit
    - enable whatever we need to enable
    - transition to long mode
    - relocate self
    - jump into kernel main


Initial offset 0x00:
    - to system setup
    - jump to rust
    - move code to different offset, randomize shit, etc etc
    - update registers.

Boot:
    - initial GDT
    - initial pages

## Pages

