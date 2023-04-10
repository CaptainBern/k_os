use core::cmp::{self, Ordering};

/// A memory region.
///
/// Regions can either be physical or virtual, it is up to the user to make clear which
/// one it is.
#[derive(Debug, PartialEq, Eq, Ord, Clone, Copy)]
pub struct Region {
    pub base: u64,
    pub length: u64,
}

impl Region {
    /// Calculate the end address of the region.
    #[inline]
    pub fn end(&self) -> u64 {
        self.base + self.length
    }

    /// Return true if `x` and `y` have any overlap.
    #[inline]
    pub fn are_overlapping(x: &Region, y: &Region) -> bool {
        (x.base <= y.end()) && (x.end() >= y.base)
    }

    /// Merge two regions.
    ///
    /// Returns [None] if the regions are not overlapping or are not of the same [RegionKind].
    #[inline]
    pub fn merge(x: &Region, y: &Region) -> Option<Region> {
        if Region::are_overlapping(x, y) {
            let base = cmp::min(x.base, y.base);
            let length = cmp::max(x.end(), y.end()) - base;
            Some(Region { base, length })
        } else {
            None
        }
    }
}

impl PartialOrd for Region {
    /// Compare `self` with `other.`
    /// * `self < other` if `(self.base < other.base) || (self.base == other.base && self.length < other.length)`
    /// * `self == other` if `self.base == other.base && self.length == other.length`
    /// * `self > other` if `(self.base > other.base) || (self.base == other.base && self.length > other.length)`
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.base.cmp(&other.base) {
            Ordering::Equal => Some(self.length.cmp(&other.length)),
            ordering => Some(ordering),
        }
    }
}

/// Memory kind.
///
/// Standard E820 memory 'types'. See the [osdev wiki](https://wiki.osdev.org/Detecting_Memory_(x86)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MemoryKind {
    Usable = 0,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    Defective,
}

/// A memory descriptor.
///
/// This is based directly on the E820 memory description as described
/// [here](https://wiki.osdev.org/Detecting_Memory_(x86)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord)]
pub struct MemoryDescriptor {
    pub region: Region,
    pub kind: MemoryKind,
}

impl MemoryDescriptor {
    /// Return true if the memory region is usable.
    ///
    /// This is just a shorthand to check if `self.kind` is [RegionKind::Usable].
    #[inline]
    pub fn is_usable(&self) -> bool {
        self.kind == MemoryKind::Usable
    }
}

impl PartialOrd for MemoryDescriptor {
    /// Memory descriptors are sorted according to the region they span.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.region.partial_cmp(&other.region)
    }
}
