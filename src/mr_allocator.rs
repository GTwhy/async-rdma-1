use crate::{LocalMemoryRegion, ProtectionDomain};
use rdma_sys::ibv_access_flags;
use std::{alloc::Layout, io, sync::Arc};

pub struct MRAllocator {
    _pd: Arc<ProtectionDomain>,
    mr: Arc<LocalMemoryRegion>,
}

impl MRAllocator {
    pub fn new(pd: Arc<ProtectionDomain>) -> Self {
        let access = ibv_access_flags::IBV_ACCESS_LOCAL_WRITE
            | ibv_access_flags::IBV_ACCESS_REMOTE_WRITE
            | ibv_access_flags::IBV_ACCESS_REMOTE_READ
            | ibv_access_flags::IBV_ACCESS_REMOTE_ATOMIC;
        let mr = Arc::new(
            pd.alloc_memory_region(Layout::from_size_align(4096 * 1024, 4096).unwrap(), access)
                .unwrap(),
        );
        Self { _pd: pd, mr }
    }

    pub fn alloc(&self, layout: Layout) -> io::Result<LocalMemoryRegion> {
        self.mr.alloc(layout)
    }

    pub fn _release(&self, _mr: LocalMemoryRegion) -> io::Result<()> {
        todo!()
    }
}
