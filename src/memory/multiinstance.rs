use sysinfo::{PidExt, ProcessExt, System, SystemExt};

#[derive(Debug)]
pub struct GameInstance {
    pub pid: u32,
    pub name: String,
    pub cmd: String,
}

pub fn detect_multiple_instances() -> Vec<GameInstance> {
    let mut system: System = System::new_all();

    // Refresh process information
    system.refresh_all();

    // Find processes named "D2R.exe"
    let matching_processes: Vec<_> = system
        .processes()
        .values()
        .filter(|process| process.name().eq_ignore_ascii_case("D2R.exe"))
        .collect();

    // Print the details of matching processes
    if matching_processes.is_empty() {
        log::info!("No processes named 'D2R.exe' found.");
        return vec![]
    } else {
        let mut game_instances = vec![];
        log::info!("Processes named 'D2R.exe':");
        for process in matching_processes {
            log::info!(
                "PID: {}, Name: {}, Status: {:?} cmd: {}",
                process.pid(),
                process.name(),
                process.status(),
                process.cmd().join(" ")
            );
            game_instances.push(GameInstance { pid: process.pid().as_u32(), name: String::from(process.name()), cmd: process.cmd().join(" ")} );
        }
        return game_instances
    }

}