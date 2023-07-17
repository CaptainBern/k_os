//! Constants related to the kernel MM.

use crate::linker;

use super::paging;

/// The size of the virtual window in which per-cpu variables (and stack) will be stored.
/// Each per-cpu block is laid out as follows:
///         +-------------------+
///         |   percpu storage  |
///         |       ...         |
///         +-------------------+
///         |    guard page     |
///         +-------------------+
///         |       stack       |
///         +-------------------+
///         |    guard page     |
///         +-------------------+
///                ...
pub const PERCPU_WINDOW_SIZE: usize =
    (linker::KERNEL_PHYS_START as usize + linker::STACK_SIZE + (linker::STACK_GUARD_SIZE * 2))
        * linker::MAX_CPUS;

pub const NUM_PERCPU_PDS: usize = paging::num_tables::<{ paging::PD_COVERAGE }>(PERCPU_WINDOW_SIZE);
pub const NUM_PERCPU_PTS: usize = paging::num_tables::<{ paging::PT_COVERAGE }>(PERCPU_WINDOW_SIZE);
pub const NUM_PHYS_PDPTS: usize =
    paging::num_tables::<{ paging::PDPT_COVERAGE }>(linker::MAX_PHYS_MEMORY);
