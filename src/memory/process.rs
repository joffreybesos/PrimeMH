use std::backtrace::{Backtrace, BacktraceStatus};
use std::fmt::Debug;
use std::mem::{size_of_val, MaybeUninit};
use std::any::type_name;
use std::ptr;
use std::mem;
use winapi;
use winapi::shared::minwindef::{DWORD, FALSE, HMODULE, LPCVOID, LPVOID, MAX_PATH, TRUE};
use winapi::shared::ntdef::NULL;
use winapi::shared::windef::{HWND, POINT, RECT};
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::WriteProcessMemory;
use winapi::um::winuser::{ClientToScreen, GetClientRect, GetDpiForWindow, GetForegroundWindow};
use winapi::um::{processthreadsapi::OpenProcess, winnt::PROCESS_ALL_ACCESS};


use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx};
use winapi::um::psapi::{EnumProcessModules, EnumProcessModulesEx, GetModuleBaseNameA, GetModuleBaseNameW, GetModuleInformation, LIST_MODULES_ALL, MODULEINFO};
use winapi::um::winnt::{
    HANDLE, MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_IMAGE, MEM_MAPPED, MEM_PRIVATE,
    PAGE_GUARD, PAGE_NOACCESS, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};

use crate::types::buffs::BuffInstance;
use crate::LOCALISATION;

use super::instance_manager::WindowInfo;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct D2RInstance {
    pub window: WindowInfo,
    pub handle: HANDLE,
    pub base_address: usize,
    pub offsets: Offsets,
    pub buff_instance: BuffInstance,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Offsets {
    pub unit_table: u64,
    pub ui_offset: u64,
    pub last_game_name: u64,
    pub hover: u64,
    pub roster: u64,
    pub panels: u64,
    pub keybindings: u64,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct D2RWindowArea {
    pub window_handle: HWND,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub left: i32,
    pub top: i32,
}

impl D2RInstance {
    pub fn new(window: &WindowInfo) -> Self {
        // https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-openprocess?redirectedfrom=MSDN
        let pid: u32 = window.pid;
        let handle: HANDLE = unsafe { OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid) };
        if handle == NULL {
            log::debug!("OpenProcess failed. Error: {:?}", std::io::Error::last_os_error());
            log::error!("{} not found\nExiting rusty reveal...", window.title);
            let localisation = LOCALISATION.lock().unwrap();
            let msg = format!("D2R: '{}' PID: {} {}\n\n{}", window.title, window.pid, localisation.get_primemh("error12"), std::io::Error::last_os_error());
            log::error!("{}", msg);
        }
        let base_address = Self::base_address(handle).unwrap();
        
        Self {
            window: (*window).clone(),
            handle,
            base_address,
            offsets: Self::find_offsets(pid),
            buff_instance: BuffInstance::new((*window).clone())
        }
    }
    
    pub fn is_window_active(&self, overlay_hwnd: u64, d2r_hwnd: Option<HWND>) -> bool {
        let hwnd: HWND = self.window.hwnd;
        let mut is_active_window = false;
        unsafe {
            if d2r_hwnd.is_some() {
                if GetForegroundWindow() == overlay_hwnd as HWND || GetForegroundWindow() == d2r_hwnd.unwrap() {
                    is_active_window = true;
                }
            } else {
                if GetForegroundWindow() == hwnd || GetForegroundWindow() == overlay_hwnd as HWND {
                    is_active_window = true;
                }
            }
        }
        is_active_window
    }


    pub fn get_window_info(&self) -> D2RWindowArea {
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let hwnd = self.window.hwnd;
        let mut position = POINT { x: 0, y: 0 };
        
        let scaling_factor: f64;
        unsafe {
            GetClientRect(hwnd, &mut rect);
            ClientToScreen(hwnd, &mut position);
            let dpi = GetDpiForWindow(hwnd);
            scaling_factor = dpi as f64 / 96.0;
        }
        D2RWindowArea {
            window_handle: hwnd,
            width: (rect.right as f64 / scaling_factor) as i32,
            height: (rect.bottom as f64 / scaling_factor) as i32,
            x: position.x,
            y: position.y,
            left: (rect.left as f64 / scaling_factor) as i32,
            top: (rect.top as f64 / scaling_factor) as i32,
        }
    }
    

    pub fn base_address(handle: HANDLE) -> Option<usize> {
        let mut maybe_hmod = MaybeUninit::<HMODULE>::uninit();
        let mut maybe_cb_needed = MaybeUninit::<DWORD>::uninit();

        let result = unsafe {
            EnumProcessModules(
                handle,
                maybe_hmod.as_mut_ptr(),
                size_of_val(&maybe_hmod) as u32,
                maybe_cb_needed.as_mut_ptr(),
            )
        };

        if result != TRUE {
            return None;
        }

        let mut base_name_vec: Vec<u8> = Vec::with_capacity(MAX_PATH);

        unsafe {
            let base_name_length = GetModuleBaseNameA(
                handle,
                maybe_hmod.assume_init(),
                base_name_vec.as_mut_ptr() as *mut _,
                base_name_vec.capacity() as u32,
            );

            base_name_vec.set_len(base_name_length as usize)
        }

        let base_name = String::from_utf8_lossy(&base_name_vec);

        if base_name.to_lowercase() == "D2R.exe".to_lowercase() {
            unsafe { Some(maybe_hmod.assume_init() as usize) }
        } else {
            None
        }
    }

    fn scan_pattern(
        pattern_name: &str,
        pid: u32,
        pattern: String,
        extra_bytes: i32,
        extra_bytes2: i32,
    ) -> u32 {
        // Parse the pattern string into bytes and mask
        let (pattern_bytes, mask) = Self::parse_pattern(&pattern);
        
        // Open the process
        let process_handle: HANDLE = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                FALSE,
                pid,
            )
        };
        
        if process_handle.is_null() {
            log::error!("Failed to open process PID {}", pid);
            return 0;
        }
        
        // Get module info for D2R.exe
        let module_info = match Self::get_module_info(process_handle, "D2R.exe") {
            Some(info) => info,
            None => {
                log::error!("Failed to get module info for D2R.exe");
                unsafe { CloseHandle(process_handle); }
                return 0;
            }
        };
        
        let base_address = module_info.lpBaseOfDll as usize;
        let end_address = base_address + module_info.SizeOfImage as usize;
        
        // Scan memory regions
        let result = Self::scan_memory_regions(
            process_handle,
            base_address,
            end_address,
            &pattern_bytes,
            &mask,
        );
        
        match result {
            Some(offset_address) => {
                // Read the offset value at the found address + extra_bytes
                let extra_bytes_address = offset_address as isize + extra_bytes as isize;
                let offset: u32 = match Self::read_memory(process_handle, extra_bytes_address as usize) {
                    Some(v) => v,
                    None => {
                        log::error!("Failed to read memory at offset address");
                        unsafe { CloseHandle(process_handle); }
                        return 0;
                    }
                };
                
                unsafe { CloseHandle(process_handle); }
                
                if extra_bytes2 > 0 {
                    ((offset_address - base_address) + extra_bytes2 as usize + offset as usize) as u32
                } else {
                    offset
                }
            }
            None => {
                unsafe { CloseHandle(process_handle); }
                panic!("Did not find {} signature\nFatal error", pattern_name);
            }
        }
    }

    /// Parse a pattern string like "48 03 C7 49 8B 8C C6" into bytes and mask
    /// Use "??" or "?" for wildcard bytes
    fn parse_pattern(pattern: &str) -> (Vec<u8>, Vec<bool>) {
        let parts: Vec<&str> = pattern.split_whitespace().collect();
        let mut bytes = Vec::with_capacity(parts.len());
        let mut mask = Vec::with_capacity(parts.len());
        
        for part in parts {
            if part == "?" || part == "??" {
                bytes.push(0);
                mask.push(false); // false = wildcard, don't match
            } else {
                bytes.push(u8::from_str_radix(part, 16).unwrap_or(0));
                mask.push(true); // true = must match
            }
        }
        
        (bytes, mask)
    }

    /// Get module information for a specific module
    fn get_module_info(process_handle: HANDLE, module_name: &str) -> Option<MODULEINFO> {
        const MAX_MODULES: usize = 1024;
        let mut modules: [HMODULE; MAX_MODULES] = [ptr::null_mut(); MAX_MODULES];
        let mut cb_needed: DWORD = 0;
        
        unsafe {
            let result = EnumProcessModulesEx(
                process_handle,
                modules.as_mut_ptr(),
                (MAX_MODULES * mem::size_of::<HMODULE>()) as DWORD,
                &mut cb_needed,
                LIST_MODULES_ALL,
            );
            
            if result == FALSE {
                return None;
            }
            
            let module_count = cb_needed as usize / mem::size_of::<HMODULE>();
            
            for i in 0..module_count {
                let mut name_buf = [0u16; MAX_PATH];
                let len = GetModuleBaseNameW(
                    process_handle,
                    modules[i],
                    name_buf.as_mut_ptr(),
                    MAX_PATH as DWORD,
                );
                
                if len > 0 {
                    let name = String::from_utf16_lossy(&name_buf[..len as usize]);
                    if name.eq_ignore_ascii_case(module_name) {
                        let mut mod_info: MODULEINFO = mem::zeroed();
                        let result = GetModuleInformation(
                            process_handle,
                            modules[i],
                            &mut mod_info,
                            mem::size_of::<MODULEINFO>() as DWORD,
                        );
                        
                        if result != FALSE {
                            return Some(mod_info);
                        }
                    }
                }
            }
        }
        
        None
    }

    /// Scan memory regions using VirtualQueryEx
    fn scan_memory_regions(
        process_handle: HANDLE,
        start_address: usize,
        end_address: usize,
        pattern: &[u8],
        mask: &[bool],
    ) -> Option<usize> {
        let mut address = start_address;
        let pattern_size = pattern.len();
        
        while address < end_address {
            let mut mem_info: MEMORY_BASIC_INFORMATION = unsafe { mem::zeroed() };
            
            let result = unsafe {
                VirtualQueryEx(
                    process_handle,
                    address as LPCVOID,
                    &mut mem_info,
                    mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            
            if result == 0 {
                break;
            }
            
            let region_size = mem_info.RegionSize;
            let region_base = mem_info.BaseAddress as usize;
            
            // Check if this region is readable and suitable for scanning
            let is_committed = mem_info.State == MEM_COMMIT;
            let is_readable = (mem_info.Protect & (PAGE_NOACCESS | PAGE_GUARD)) == 0;
            let is_scannable_type = mem_info.Type == MEM_MAPPED 
                || mem_info.Type == MEM_PRIVATE
                || mem_info.Type == MEM_IMAGE;
            
            if is_committed && is_readable && is_scannable_type && region_size >= pattern_size {
                // Read the entire region
                if let Some(buffer) = Self::read_region(process_handle, region_base, region_size) {
                    // Scan for pattern in this region
                    if let Some(offset) = Self::pattern_scan(&buffer, pattern, mask) {
                        return Some(region_base + offset);
                    }
                }
            }
            
            // Move to next region
            address = region_base + region_size;
        }
        
        None
    }

    /// Read a memory region into a buffer
    fn read_region(process_handle: HANDLE, address: usize, size: usize) -> Option<Vec<u8>> {
        let mut buffer = vec![0u8; size];
        let mut bytes_read: usize = 0;
        
        let result = unsafe {
            ReadProcessMemory(
                process_handle,
                address as LPCVOID,
                buffer.as_mut_ptr() as LPVOID,
                size,
                &mut bytes_read,
            )
        };
        
        if result != FALSE && bytes_read > 0 {
            buffer.truncate(bytes_read);
            Some(buffer)
        } else {
            None
        }
    }

    /// Read a value from memory
    fn read_memory<T: Copy>(process_handle: HANDLE, address: usize) -> Option<T> {
        let mut value: T = unsafe { mem::zeroed() };
        let mut bytes_read: usize = 0;
        
        let result = unsafe {
            ReadProcessMemory(
                process_handle,
                address as LPCVOID,
                &mut value as *mut T as LPVOID,
                mem::size_of::<T>(),
                &mut bytes_read,
            )
        };
        
        if result != FALSE && bytes_read == mem::size_of::<T>() {
            Some(value)
        } else {
            None
        }
    }

    /// Scan a buffer for a pattern with mask
    fn pattern_scan(buffer: &[u8], pattern: &[u8], mask: &[bool]) -> Option<usize> {
        if buffer.len() < pattern.len() {
            return None;
        }
        
        let max_offset = buffer.len() - pattern.len();
        
        'outer: for i in 0..=max_offset {
            for j in 0..pattern.len() {
                // If mask is true, byte must match; if false, it's a wildcard
                if mask[j] && buffer[i + j] != pattern[j] {
                    continue 'outer;
                }
            }
            return Some(i);
        }
        
        None
    }
    pub fn find_offsets(pid: u32) -> Offsets {

        let pattern = String::from("48 03 C7 49 8B 8C C6");
        let unit_table: u32 = Self::scan_pattern("Unit table", pid, pattern, 7, 0);
        // let unit_table = 0x1D95AF0;
        log::debug!("Unit offset 0x{:02x}", unit_table);
    
        let pattern = String::from("40 84 ed 0f 94 05 4b");
        let ui_offset = Self::scan_pattern("UI offset", pid, pattern, 6, 10) - 10;
        // let ui_offset = 0x1DA57DA;
        log::debug!("UI offset 0x{:02x}", ui_offset);
    
        let pattern = String::from("C6 84 C2 ? ? ? ? ? 48 8B 74 24 ?");
        let hover = Self::scan_pattern("Hover", pid, pattern, 3, 0) - 1;
        // let hover = 0x1CE8400;
        log::debug!("Hover offset 0x{:02x}", hover);
    
        let pattern = String::from("?? 45 33 D2 4D 8B");
        let roster = Self::scan_pattern("Roster", pid, pattern, -3, 1);
        // let roster = 0x1DABD60;
        log::debug!("Roster offset 0x{:02x}", roster);
    
        let pattern = String::from("48 89 05 ? ? ? ? 48 85 DB 74 1E");
        let panels = Self::scan_pattern("Panels", pid, pattern, 3, 7);
        // let panels = 0x1D00968;
        log::debug!("Panel offset 0x{:02x}", panels);

        let pattern = String::from("02 00 00 00 ? ? 00 00 00 00 03 00 00 00 ? ? 01 00 00 00");
        let keybindings = Self::scan_pattern("Keybindings", pid, pattern, 0, 0x158C);
        // let keybindings = 0x18C2894;
        log::debug!("Keybindings offset 0x{:02x}", keybindings);
        
        Offsets {
            unit_table: unit_table as u64,
            ui_offset: (ui_offset - 0xA) as u64,
            last_game_name: 0x24D5A90,
            hover: hover as u64,
            roster: roster as u64,
            panels: panels as u64,
            keybindings: keybindings as u64,
        }
    }

    pub fn read_mem_offset<T: Default + Debug>(&self, offset: u64) -> T {
        use winapi::um::memoryapi::ReadProcessMemory;

        let mut ret: T = Default::default();
        if offset == 0 || offset == 1 {
            return ret;
        }
        let address = offset as u64 + self.base_address as u64;

        unsafe {
            let rpm_return = ReadProcessMemory(
                self.handle,
                address as *mut _,
                &mut ret as *mut T as LPVOID,
                std::mem::size_of::<T>(),
                NULL as *mut usize,
            );
            if rpm_return == FALSE {
                let caller = get_caller();
                log::debug!("ReadProcessMemory read_mem_offset failed. Error: {:?}, ptr: {:?}, type: {}, caller: {}", std::io::Error::last_os_error(), &address, type_name::<T>(), caller);    
            }
        }
        ret
    }

    pub fn read_mem<T: Default + Debug>(&self, address: u64) -> T {
        use winapi::um::memoryapi::ReadProcessMemory;   

        let mut ret: T = Default::default();
        if address == 0 || address == 1 {
            return ret;
        }

        unsafe {
            let rpm_return = ReadProcessMemory(
                self.handle,
                address as *mut _,
                &mut ret as *mut T as LPVOID,
                std::mem::size_of::<T>(),
                NULL as *mut usize,
            );
            if rpm_return == FALSE {
                let caller = get_caller();
                log::debug!("ReadProcessMemory failed. Error: {:?}, ptr: {:?}, type: {}, caller: {}", std::io::Error::last_os_error(), &address, type_name::<T>(), caller);    
            }
        }
        ret
    }

    pub fn _write_mem<T: Copy>(&self, address: u64, value: T) -> bool {
        if address == 0 || address == 1 {
            return false;
        }

        unsafe {
            let result = WriteProcessMemory(
                self.handle,
                address as *mut _,
                &value as *const T as LPVOID,
                std::mem::size_of::<T>(),
                NULL as *mut usize,
            );

            if result == 0 {
                log::debug!(
                    "WriteProcessMemory failed. Error: {:?}, ptr: {:?}, type: {}",
                    std::io::Error::last_os_error(),
                    &address,
                    std::any::type_name::<T>()
                );
                return false;
            }
        }

        true
    }

    pub fn parse_arr_to_string(&self, bytes: &[u8]) -> String {
        let mut fixed_string: Vec<u8> = vec![];
        for b in bytes {
            if *b == 0 {
                break;
            }
            fixed_string.push(b.clone());
        }
        unsafe { String::from_utf8_unchecked(fixed_string) }
    }

    #[allow(dead_code)]
    pub fn close(&self) {
        unsafe { CloseHandle(self.handle) };
    }
}

fn get_caller() -> String {
    let backtrace = Backtrace::capture();
    if backtrace.status() == BacktraceStatus::Captured {
        let backtrace_string = format!("{:?}", backtrace);
        let entries = backtrace_string.split("{ fn: ").collect::<Vec<&str>>();
        // log::info!("{:?}", entries);
        let mut calling_func = "";
        for i in 0..entries.len() - 1 {
            if entries[i].contains("PrimeMH::memory::process::D2RInstance::read_mem") {
                calling_func = entries[i + 1];
            }
        }
        return calling_func.to_string();
    }
    return String::new()
}