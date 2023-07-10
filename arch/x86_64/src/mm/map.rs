use super::paging::{
    is_aligned, PDEFlags, PDPTEFlags, PML4EFlags, PTEFlags, BASE_PAGE, GIGA_PAGE, MEGA_PAGE, PD,
    PDE, PDPT, PDPTE, PML4, PML4E, PT, PTE,
};

/// Handle flags on mapped entries.
#[derive(Debug)]
pub enum Flags<T> {
    /// Preserve whatever flags were set. This includes keeping the flags
    /// clear if none were set prior.
    Preserve,

    /// Initialise the flags to the given value if none were set.
    Init(T),

    /// Set the flags to the given value. Overrides any flags set prior.
    Set(T),

    /// Preserve existing flags but make sure the given flags are set.
    Enable(T),

    /// Preserve existing flags, but disable the given flags.
    Disable(T),
}

macro_rules! flags {
    ($old_flags:expr, $flags:ident) => {{
        let old_flags = $old_flags;
        let new_flags = match $flags {
            Flags::Preserve => old_flags,
            Flags::Init(flags) => {
                if old_flags.is_empty() {
                    flags
                } else {
                    old_flags
                }
            }
            Flags::Set(flags) => flags,
            Flags::Enable(flags) => old_flags | flags,
            Flags::Disable(flags) => old_flags & !flags,
        };

        new_flags
    }};
}

/// A helper to simplify building static page tables.
///
/// * `LINK_OFFSET` - The offset at which the binary is linked. This value is used to calculate
///                    the physical address of tables.
#[derive(Debug)]
pub struct Mapper<'a, const LINK_OFFSET: usize> {
    top: &'a mut PML4,
}

impl<'a, const LINK_OFFSET: usize> Mapper<'a, LINK_OFFSET> {
    pub fn new(top: &'a mut PML4) -> Self {
        Self { top }
    }

    /// Map a 512G memory range.
    pub fn pdpt<'b>(
        &mut self,
        pml4_idx: usize,
        pdpt: &'b mut PDPT,
        flags: Flags<PML4EFlags>,
    ) -> PdptMapper<'b, LINK_OFFSET> {
        assert!(pml4_idx < 512);
        assert!(is_aligned::<{ BASE_PAGE }>(pdpt.table.as_ptr() as u64));
        self.top.table[pml4_idx] = PML4E::new(
            pdpt.table.as_ptr() as u64 - LINK_OFFSET as u64,
            flags!(self.top.table[pml4_idx].flags(), flags),
        );

        PdptMapper { pdpt }
    }
}

/// A mapper that can map a 512G range of memory using 1G pages or 2M/4K pages with a
/// provided PD.
#[derive(Debug)]
pub struct PdptMapper<'a, const LINK_OFFSET: usize> {
    pdpt: &'a mut PDPT,
}

impl<'a, const LINK_OFFSET: usize> PdptMapper<'a, LINK_OFFSET> {
    /// Map a 1G page.
    ///
    /// It is up to the caller to provide the apprioriate flags (P, PS, etc).
    pub fn map(&mut self, pdpt_idx: usize, frame: u64, flags: Flags<PDPTEFlags>) {
        assert!(pdpt_idx < 512);
        assert!(is_aligned::<{ GIGA_PAGE }>(frame));

        self.pdpt.table[pdpt_idx] =
            PDPTE::new(frame, flags!(self.pdpt.table[pdpt_idx].flags(), flags));
    }

    /// Map a 1G memory range.
    pub fn pd<'b>(
        &mut self,
        pdpt_idx: usize,
        pd: &'b mut PD,
        flags: Flags<PDPTEFlags>,
    ) -> PdMapper<'b, LINK_OFFSET> {
        assert!(pdpt_idx < 512);
        assert!(is_aligned::<{ BASE_PAGE }>(pd.table.as_ptr() as u64));

        self.pdpt.table[pdpt_idx] = PDPTE::new(
            pd.table.as_ptr() as u64 - LINK_OFFSET as u64,
            flags!(self.pdpt.table[pdpt_idx].flags(), flags),
        );

        PdMapper { pd }
    }
}

/// A mapper that can map a 1G range of memory using either 2M pages, or 4K pages using
/// a provided PT.
#[derive(Debug)]
pub struct PdMapper<'a, const LINK_OFFSET: usize> {
    pd: &'a mut PD,
}

impl<'a, const LINK_OFFSET: usize> PdMapper<'a, LINK_OFFSET> {
    /// Map a 2M page.
    ///
    /// It is up to the caller to provide the apprioriate flags (P, PS, etc).
    pub fn map(&mut self, pd_idx: usize, frame: u64, flags: Flags<PDEFlags>) {
        assert!(pd_idx < 512);
        assert!(is_aligned::<{ MEGA_PAGE }>(frame));

        self.pd.table[pd_idx] = PDE::new(frame, flags!(self.pd.table[pd_idx].flags(), flags));
    }

    /// Map a 2M memory range.
    pub fn pt<'b>(
        &mut self,
        pd_idx: usize,
        pt: &'b mut PT,
        flags: Flags<PDEFlags>,
    ) -> PtMapper<'b, LINK_OFFSET> {
        assert!(pd_idx < 512);
        assert!(is_aligned::<{ BASE_PAGE }>(pt.table.as_ptr() as u64));

        self.pd.table[pd_idx] = PDE::new(
            pt.table.as_ptr() as u64 - LINK_OFFSET as u64,
            flags!(self.pd.table[pd_idx].flags(), flags),
        );

        PtMapper { pt }
    }
}

/// A mapper that can map a 2M range of memory using 4K pages.
#[derive(Debug)]
pub struct PtMapper<'a, const LINK_OFFSET: usize> {
    pt: &'a mut PT,
}

impl<'a, const LINK_OFFSET: usize> PtMapper<'a, LINK_OFFSET> {
    /// Map a 4K page.
    ///
    /// It is up to the caller to provide the apprioriate flags (P, etc).
    pub fn map(&mut self, pt_idx: usize, frame: u64, flags: Flags<PTEFlags>) {
        assert!(pt_idx < 512);
        assert!(is_aligned::<{ BASE_PAGE }>(frame));

        self.pt.table[pt_idx] = PTE::new(frame, flags!(self.pt.table[pt_idx].flags(), flags));
    }
}
