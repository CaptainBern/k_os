# Kernel memory layout

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
| 0xffffff8000000000 | 0xffffff80ffffffff | 4G     | per cpu data (with a limit of 16MB, 8G supports up to 512 cores) |
|                    |                    | 506GB  | unused gap                                                       |
| 0xffffffff80000000 | 0xffffffff9fffffff | 512MB  | kernel text/data                                                 |
| 0xffffffffa0000000 | 0xffffffffbfffffff | 512MB  | unused gap                                                       |
| 0xffffffffc0000000 | 0xffffffffffffffff | 1G     | kernel devices                                                   |

## Allocations and kernel heap

The kernel itself performs as little memory management as possible. The only
allocations happening at runtime (in kernel space) is the process of setting up
per-cpu variables and stack space. k_os, like seL4, is a single-kernel-stack-
per-core kernel.

Most of he memory management can be tuned at compile-time to match the target
system as close as possible. By doing this, k's footprint can be lowered to the
bare minimum required to run the system.