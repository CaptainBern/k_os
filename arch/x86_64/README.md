
# TODO

initialise boot cpu, and additional processors if necessary.

use %fs for thread-local storage -> no ???
use %gs for kernel stack/TCB
use ASPACE_LOCAL_START for per-cpu storage.

Enter Rust with simple mappings, here switch to proper pages and all

# Memory map

- Use #[thread_local] for per-cpu page tables

================================================================================================
start_addr       | offset | end_addr          | size | description 
=================================================================================================
ffffffff80000000 |  -2G   | ffffffff9fffffff  | 512M | kernel text, mapped to physical address 0
ffffffffa0000000 | -1.5G  | 
