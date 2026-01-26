use std::backtrace::{Backtrace, BacktraceStatus};
use std::fmt::Debug;
use std::mem::{size_of_val, MaybeUninit};
use std::any::type_name;
use proc_mem::ProcMemError;
use winapi;
use winapi::shared::minwindef::{DWORD, FALSE, HMODULE, LPVOID, MAX_PATH, TRUE};
use winapi::shared::ntdef::HANDLE;
use winapi::shared::ntdef::NULL;
use winapi::shared::windef::{HWND, POINT, RECT};
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::WriteProcessMemory;
use winapi::um::psapi::{EnumProcessModules, GetModuleBaseNameA};
use winapi::um::winuser::{ClientToScreen, GetClientRect, GetDpiForWindow, GetForegroundWindow};
use winapi::um::{processthreadsapi::OpenProcess, winnt::PROCESS_ALL_ACCESS};

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
    pub expansion: u64,
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

    
    fn scan_pattern(pattern_name: &str, pid: u32, pattern: String, extra_bytes: i32, extra_bytes2: i32) -> u32 {
        use proc_mem::{Process, Signature};
        
        let some_game = Process::with_pid(pid);
        let game = match some_game {
            Ok(s) => s,
            Err(err) => {
                let localisation = LOCALISATION.lock().unwrap();
                let msg = format!("{} PID {}\n{:?}", localisation.get_primemh("error13"), pid, err);
                log::error!("{}", msg);
                return 0;
            },
        };
        let module = game.module("D2R.exe").unwrap();
        let lp_signature = Signature {
            name: "LocalPlayer".to_owned(),
            pattern,
            offsets: vec![],
            extra: 0,
            relative: false,
            rip_relative: false,
            rip_offset: 0,
        };
        let lp_address: Result<usize, ProcMemError> = module.find_signature(&lp_signature);
        let offset_address = match lp_address {
            Ok(a) => a,
            Err(err) => {
                panic!("Did not find {} signature: {:?}\nFatal error", pattern_name, err);
            },
        };
        let extra_bytes_address = offset_address as isize + extra_bytes as isize;
        let offset = game.read_mem(extra_bytes_address as usize).unwrap();
        if extra_bytes2 > 0 {
            return ((offset_address - game.process_base_address) + extra_bytes2 as usize + offset as usize) as u32;
        } else {
            return offset;
        }
    }

    pub fn find_offsets(pid: u32) -> Offsets {

        let pattern = String::from("48 03 C7 49 8B 8C C6");
        let unit_table = Self::scan_pattern("Unit table", pid, pattern, 7, 0);
        // let unit_table = 0x1D95AF0;
        log::debug!("Unit offset 0x{:02x}", unit_table);
    
        let pattern = String::from("40 84 ed 0f 94 05 4b");
        let ui_offset = Self::scan_pattern("UI offset", pid, pattern, 6, 10) - 10;
        // let ui_offset = 0x1DA57DA;
        log::debug!("UI offset 0x{:02x}", ui_offset);
    
        let pattern = String::from("48 8B 05 ? ? ? ? 48 8B D9 F3 0F 10 50 ?");
        let expansion = Self::scan_pattern("Expansion", pid, pattern, 3, 7);
        // let expansion = 0x1CE78D0;
        log::debug!("Exp offset 0x{:02x}", expansion);
    
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
            expansion: expansion as u64,
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