use super::paging::{
    is_aligned, PDEFlags, PDPTEFlags, PML4EFlags, PTEFlags, BASE_PAGE, GIGA_PAGE, MEGA_PAGE, PD,
    PDE, PDPT, PDPTE, PML4, PML4E, PT, PTE,
};

/// A helper to simplify building static page tables.
///
/// * `LINK_OFFSET` - The offset at which the binary is linked. This value is used to calculate
///                    the physical address of tables.
#[derive(Debug)]
pub struct Mapper<const LINK_OFFSET: usize> {
    top: &'static mut PML4,
}

impl<const LINK_OFFSET: usize> Mapper<LINK_OFFSET> {
    pub fn new(top: &'static mut PML4) -> Self {
        Self { top }
    }

    /// Map a 512G memory range.
    pub fn pdpt(
        &mut self,
        pml4_idx: usize,
        pdpt: &'static mut PDPT,
        flags: PML4EFlags,
    ) -> PdptMapper<LINK_OFFSET> {
        assert!(pml4_idx < 512);
        assert!(is_aligned::<{ BASE_PAGE }>(pdpt.table.as_ptr() as u64));
        self.top.table[pml4_idx] =
            PML4E::new(pdpt.table.as_ptr() as u64 - LINK_OFFSET as u64, flags);

        PdptMapper {
            _mapper: self,
            pdpt,
        }
    }
}

/// A mapper that can map a 512G range of memory using 1G pages or 2M/4K pages with a
/// provided PD.
#[derive(Debug)]
pub struct PdptMapper<'a, const LINK_OFFSET: usize> {
    _mapper: &'a Mapper<LINK_OFFSET>,
    pdpt: &'static mut PDPT,
}

impl<'a, const LINK_OFFSET: usize> PdptMapper<'a, LINK_OFFSET> {
    /// Map a 1G page.
    ///
    /// It is up to the caller to provide the apprioriate flags (P, PS, etc).
    pub fn map(&mut self, pdpt_idx: usize, frame: u64, flags: PDPTEFlags) {
        assert!(pdpt_idx < 512);
        assert!(is_aligned::<{ GIGA_PAGE }>(frame));
        self.pdpt.table[pdpt_idx] = PDPTE::new(frame, flags);
    }

    /// Map a 1G memory range.
    pub fn pd(
        &mut self,
        pdpt_idx: usize,
        pd: &'static mut PD,
        flags: PDPTEFlags,
    ) -> PdMapper<LINK_OFFSET> {
        assert!(pdpt_idx < 512);
        assert!(is_aligned::<{ BASE_PAGE }>(pd.table.as_ptr() as u64));
        self.pdpt.table[pdpt_idx] =
            PDPTE::new(pd.table.as_ptr() as u64 - LINK_OFFSET as u64, flags);

        PdMapper { _mapper: self, pd }
    }
}

/// A mapper that can map a 1G range of memory using either 2M pages, or 4K pages using
/// a provided PT.
#[derive(Debug)]
pub struct PdMapper<'a, const LINK_OFFSET: usize> {
    _mapper: &'a PdptMapper<'a, LINK_OFFSET>,
    pd: &'static mut PD,
}

impl<'a, const LINK_OFFSET: usize> PdMapper<'a, LINK_OFFSET> {
    /// Map a 2M page.
    ///
    /// It is up to the caller to provide the apprioriate flags (P, PS, etc).
    pub fn map(&mut self, pd_idx: usize, frame: u64, flags: PDEFlags) {
        assert!(pd_idx < 512);
        assert!(is_aligned::<{ MEGA_PAGE }>(frame));
        self.pd.table[pd_idx] = PDE::new(frame, flags);
    }

    /// Map a 2M memory range.
    pub fn pt(
        &mut self,
        pd_idx: usize,
        pt: &'static mut PT,
        flags: PDEFlags,
    ) -> PtMapper<LINK_OFFSET> {
        assert!(pd_idx < 512);
        assert!(is_aligned::<{ BASE_PAGE }>(pt.table.as_ptr() as u64));
        self.pd.table[pd_idx] = PDE::new(pt.table.as_ptr() as u64 - LINK_OFFSET as u64, flags);

        PtMapper { _mapper: self, pt }
    }
}

/// A mapper that can map a 2M range of memory using 4K pages.
#[derive(Debug)]
pub struct PtMapper<'a, const LINK_OFFSET: usize> {
    _mapper: &'a PdMapper<'a, LINK_OFFSET>,
    pt: &'static mut PT,
}

impl<'a, const LINK_OFFSET: usize> PtMapper<'a, LINK_OFFSET> {
    /// Map a 4K page.
    ///
    /// It is up to the caller to provide the apprioriate flags (P, etc).
    pub fn map(&mut self, pt_idx: usize, frame: u64, flags: PTEFlags) {
        assert!(pt_idx < 512);
        assert!(is_aligned::<{ BASE_PAGE }>(frame));
        self.pt.table[pt_idx] = PTE::new(frame, flags);
    }
}
