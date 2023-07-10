use core::cmp::Ordering;

use heapless::{binary_heap::Min, BinaryHeap, Vec};
use itertools::Itertools;

use crate::linker;

use super::{
    desc::{MemoryDescriptor, Region},
    paging,
};

const LOWERMEM_END: u64 = paging::MEGABYTE as u64;

pub type Result<T> = core::result::Result<T, MemoryError>;

#[derive(Debug, Clone, Copy)]
pub enum MemoryError {
    Oom,
}

/// Keeps track of usable memory.
#[derive(Debug)]
pub struct Memory<const NUM_REGIONS: usize> {
    /// An heap of usable memory regions.
    ///
    /// Note that the heap is `+ 1` in size here. When we parse the region
    /// containing the kernel, we need to split it in two, which would create
    /// an extra entry.
    mem: BinaryHeap<Region, Min, { crate::MAX_MEM_REGIONS + 1 }>,
}

impl<const NUM_REGIONS: usize> Memory<NUM_REGIONS> {
    /// Initialise the free memory using the given descriptors.
    ///
    /// Any descriptor for memory within the first 1M will be discarded. Overlapping regions
    /// will be merged, and every region is 4K aligned. Memory occupied by the kernel will
    /// not be included, so any frame retrieved with [`next`] is guaranteed to be free.
    pub fn new(descriptors: &Vec<MemoryDescriptor, { crate::MAX_MEM_REGIONS }>) -> Self {
        let kernel_region: Region = Region {
            base: linker::KERNEL_PHYS_START,
            length: (linker::_end() - linker::VIRT_OFFSET) as usize,
        };
        let kernel_region = kernel_region.align::<{ paging::BASE_PAGE }>().unwrap();

        let mut mem: BinaryHeap<Region, Min, { crate::MAX_MEM_REGIONS + 1 }> = BinaryHeap::new();
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
            .filter_map(|region| region.align::<{ paging::BASE_PAGE }>())
            .filter_map(|region| {
                if region.base < LOWERMEM_END {
                    if region.end() < LOWERMEM_END {
                        None
                    } else {
                        Some(Region {
                            base: LOWERMEM_END,
                            length: (region.end() - LOWERMEM_END) as usize,
                        })
                    }
                } else {
                    Some(region)
                }
            })
            .for_each(|region| {
                let final_region = {
                    if Region::are_overlapping(&region, &kernel_region) {
                        if region.end() <= kernel_region.end() {
                            if region.base < kernel_region.base {
                                // Region end lies before kernel end, region base lies before
                                // kernel base, since we overlap that means the region end falls
                                // partly inside the kernel region.
                                Some(Ok(Region {
                                    base: region.base,
                                    length: (kernel_region.base - region.base) as usize,
                                }))
                            } else {
                                // The region base falls inside the kernel region, as does the end,
                                // so we discard it.
                                None
                            }
                        } else {
                            if region.base < kernel_region.base {
                                // The region end falls beyond the kernel region end, and region
                                // base falls before the kernel region base. That means the region
                                // is split in half by the kernel region.
                                let before = Region {
                                    base: region.base,
                                    length: (kernel_region.base - region.base) as usize,
                                };

                                let after = Region {
                                    base: kernel_region.end(),
                                    length: (region.end() - kernel_region.end()) as usize,
                                };

                                Some(Err((before, after)))
                            } else {
                                Some(Ok(Region {
                                    base: kernel_region.end(),
                                    length: (region.end() - kernel_region.end()) as usize,
                                }))
                            }
                        }
                    } else {
                        Some(Ok(region))
                    }
                };

                if let Some(region) = final_region {
                    // Safety: we have at most `NUM_REGIONS` to work with. Any overlapping or
                    // duplicates are merged first. So, there can only be a single occurance
                    // of a region splitting in two because the kernel lies in the middle of it.
                    // Since the heap we use is `NUM_REGIONS + 1`, it should not overflow.
                    match region {
                        Ok(region) => unsafe { mem.push_unchecked(region) },
                        Err((before, after)) => unsafe {
                            mem.push_unchecked(before);
                            mem.push_unchecked(after);
                        },
                    }
                }
            });

        Memory { mem }
    }

    /// Return the maximum length of a contiguous chunk of memory.
    ///
    /// It does *not* return the total remaining memory!
    pub fn max(&self) -> usize {
        self.mem.peek().map_or(0, |r| r.length)
    }

    /// Peek the next frame.
    pub fn peek(&self) -> Result<u64> {
        if self.max() < paging::BASE_PAGE {
            Err(MemoryError::Oom)
        } else {
            self.mem
                .peek()
                .map(|region| region.base)
                .ok_or(MemoryError::Oom)
        }
    }

    /// Return the next 4K block.
    pub fn next(&mut self) -> Result<u64> {
        match self.max().cmp(&paging::BASE_PAGE) {
            Ordering::Less => {
                // Every region on the heap is 4K aligned and popped when empty.
                // Hence if we ever come across this situation, either the heap is empty
                // or there's a malformed region, which is a bug.
                Err(MemoryError::Oom)
            }
            Ordering::Equal => self
                .mem
                .pop()
                .map(|region| region.base)
                .ok_or(MemoryError::Oom),
            Ordering::Greater => {
                let mut region = self.mem.peek_mut().unwrap();
                let base = region.base;
                region.base += paging::BASE_PAGE as u64;
                region.length -= paging::BASE_PAGE;
                Ok(base)
            }
        }
    }
}
