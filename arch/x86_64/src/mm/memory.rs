use core::cmp;

use heapless::{binary_heap::Min, BinaryHeap, Vec};
use itertools::Itertools;

use crate::linker;

use super::{
    desc::{MemoryDescriptor, Region},
    paging::{self, align_up, is_aligned},
};

pub type Result<T> = core::result::Result<T, MemoryError>;

#[derive(Debug, Clone, Copy)]
pub enum MemoryError {
    Oom,
}

/// Keeps track of usable memory.
#[derive(Debug)]
pub struct Memory<const NUM_REGIONS: usize> {
    /// An heap of usable memory regions.
    mem: BinaryHeap<Region, Min, { crate::MAX_MEM_REGIONS }>,
}

impl<const NUM_REGIONS: usize> Memory<NUM_REGIONS> {
    /// Initialise the free memory using the given descriptors.
    pub fn new(descriptors: &Vec<MemoryDescriptor, { crate::MAX_MEM_REGIONS }>) -> Self {
        let kernel_region: Region = Region {
            base: 0,
            length: (linker::_end() - linker::VIRT_OFFSET) as usize,
        };

        let mut mem: BinaryHeap<Region, Min, { crate::MAX_MEM_REGIONS }> = BinaryHeap::new();
        let mut descriptors = descriptors.clone();

        // For the coalescing we need the regions to be sorted.
        descriptors.sort_unstable();

        descriptors
            .iter()
            .filter_map(|desc| {
                if desc.is_usable() {
                    Some(desc.region)
                } else {
                    None
                }
            })
            .coalesce(|left, right| {
                if Region::are_overlapping(&left, &right) {
                    Ok(Region::merge(&left, &right).unwrap())
                } else {
                    Err((left, right))
                }
            })
            .filter_map(|region| {
                // The provided regions *should* be properly aligned, but y'know, just in case
                // we get a weird bios.
                if !is_aligned::<{ paging::BASE_PAGE }>(region.base) {
                    let diff = align_up::<{ paging::BASE_PAGE }>(region.base) - region.base;
                    if (region.length as u64 - diff) > 0 {
                        Some(Region {
                            base: region.base + diff,
                            length: region.length - diff as usize,
                        })
                    } else {
                        None
                    }
                } else {
                    Some(region)
                }
            })
            .filter_map(|region| {
                if region.end() <= kernel_region.end() {
                    None
                } else if region.base <= kernel_region.end() {
                    Some(Region {
                        base: kernel_region.end(),
                        length: (region.end() - kernel_region.end()) as usize,
                    })
                } else {
                    Some(region)
                }
            })
            .for_each(|region| unsafe { mem.push_unchecked(region) }); // SAFETY: 'mem' and 'descriptors' have the same length.

        Memory { mem }
    }

    /// Return the maximum length of a contiguous chunk of memory.
    ///
    /// It does *not* return the total remaining memory!
    pub fn max(&self) -> usize {
        self.mem.peek().map_or(0, |r| r.length)
    }

    /// Return the next 4K block.
    pub fn next(&mut self) -> Result<u64> {
        match self.max().cmp(&paging::BASE_PAGE) {
            cmp::Ordering::Less => {
                if let Some(_) = self.mem.pop() {
                    self.next()
                } else {
                    Err(MemoryError::Oom)
                }
            }
            cmp::Ordering::Equal => self
                .mem
                .pop()
                .map(|region| region.base)
                .ok_or(MemoryError::Oom),
            cmp::Ordering::Greater => {
                let mut region = self.mem.peek_mut().unwrap();
                let base = region.base;
                region.base += paging::BASE_PAGE as u64;
                region.length -= paging::BASE_PAGE;
                Ok(base)
            }
        }
    }
}
