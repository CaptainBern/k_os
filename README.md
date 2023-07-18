# k_os

A microkernel written in Rust.

## Dependencies

Build dependencies:
- lld
- rust (nightly)

Required dependencies for running inside Qemu:
- Qemu (x86_64)
- mtools
- libisoburn
- grub (grub-mkrescue is used to build the ISO)

## Building

To build:
```
$ cargo xtask build
```

To run in qemu:
```
$ cargo xtask run
```