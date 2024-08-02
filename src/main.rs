#![windows_subsystem = "windows"]

#[allow(unused)]
#[macro_use]
extern crate log;

use std::{fs::File, io::Write};
use gui::ui::start_ui;
use logger::configure_logging;

mod gui;
mod localisation;
mod mapgeneration;
mod memory;
mod settings;
#[path = "memory/types/mod.rs"]
mod types;
mod logger;

pub const SETTINGS_FILE: &str = "settings.toml";
pub const ITEM_FILTER_FILE: &str = "itemfilter.yml";


fn main() {
    
    use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS}; 
    unsafe { 
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
    configure_logging();
    log::info!("Configured logging");
    let icon = include_bytes!("./gui/images/primemh.png");
    let mut f = File::create("primemh.png").unwrap();
    f.write_all(icon.as_slice()).unwrap();
    log::info!("Added Icon");
    log::info!("Starting UI...");
    start_ui().unwrap();
}
