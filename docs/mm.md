# Kenrel memory layout

## About paging

Paging allows us to easily split up the physical memory in many virtual
address spaces, with overlapping addresses. Address translation (in 4-level
paging) looks as follows:

```
11111111 11111111 11111111 11111111 11111111 11111111 11111111 11111111
                  |        ||        ||        ||        ||___________|
                  |        ||        ||        ||________| offset (4K)|
                  |        ||        ||________|| table               |
                  |        ||________||directory|                     |
                  |________|  dir ptr |         |                     |
                      PML4            |         |_____________________|
                                      |                 offset (2M)   |
                                      |                               |
                                      |_______________________________|
                                                 offset (1G)
```

Addresses must be canonical, which means the 9 most significant bits must be 1
or 0. Any address starting with 1 is considered a kernel address, even though
most of the kernel address space is unused.

## Kernel address space

The kernel uses 4-level paging, which allows up to 64TB of physical memory. The
layout of the kernel address space is as follows:

| start              | end                | size   | description                                                      |
|--------------------|--------------------|--------|------------------------------------------------------------------|
| 0x0000000000000000 | 0x00007fffffffffff | 128TB  | userspace (47 bits)                                              |
| 0xffff800000000000 | 0xffffbfffffffffff | 64TB   | direct mapping of all physical memory                            |
|                    |                    | 63.5TB | unused gap                                                       |
| 0xffffff8000000000 | 0xffffff81ffffffff | 8G     | per cpu data (with a limit of 16MB, 8G supports up to 512 cores) |
|                    |                    | 502GB  | unused gap                                                       |
| 0xffffffff80000000 | 0xffffffff9fffffff | 512MB  | kernel text/data                                                 |
|                    |                    | 1.5GB  | unused gap                                                       |

## Allocations and kernel heap

Memory management is pretty challenging. I opted to let a userspace server
handle any memory allocations and management. Of course the kernel is
responsible for setting itself up. This requires a small amount of memory
management, mainly just setting up the kernel address space and switching
to it.

For now, the kernel does perform some memory allocation, namely to setup
the per-cpu data. Ideally, all this work should be done by a bootloader or
similar, after which the kernel is started, but I opted to just use Grub
for now.