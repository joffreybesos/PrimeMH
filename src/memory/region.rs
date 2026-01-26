use winapi::um::memoryapi::VirtualQueryEx;
use winapi::um::winnt::{MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_NOACCESS, PAGE_GUARD};
use winapi::shared::minwindef::{DWORD};
use winapi::shared::ntdef::HANDLE;

pub struct MemoryRegion {
    pub base_address: usize,
    pub region_size: usize,
    pub state: DWORD,
    pub protect: DWORD,
    pub region_type: DWORD,
}

pub fn query_region(handle: HANDLE, address: usize) -> Option<MemoryRegion> {
    let mut mbi = MEMORY_BASIC_INFORMATION {
        BaseAddress: 0 as _,
        AllocationBase: 0 as _,
        AllocationProtect: 0,
        RegionSize: 0,
        State: 0,
        Protect: 0,
        Type: 0,
    };
    let result = unsafe {
        VirtualQueryEx(
            handle,
            address as _,
            &mut mbi,
            std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    if result == 0 {
        return None;
    }
    Some(MemoryRegion {
        base_address: mbi.BaseAddress as usize,
        region_size: mbi.RegionSize,
        state: mbi.State,
        protect: mbi.Protect,
        region_type: mbi.Type,
    })
}
