extern crate winapi;
use std::os::windows::process::CommandExt;


#[derive(Debug)]
pub struct D2RInstanceDetails {
    pub pid: u32,
    pub title: String
}

pub fn detect_multiple_instances() -> Vec<D2RInstanceDetails> {
    let mut game_instances = vec![];
    let task_list: String = tasklist();
    let mut rdr = csv::Reader::from_reader(task_list.as_bytes());

    for result in rdr.records() {
        let record = match result {
            Ok(it) => it,
            Err(_) => todo!(),
        };
        
        let pid = record[1].parse::<u32>().unwrap();
        let title = record[8].parse::<String>().unwrap();
        game_instances.push(D2RInstanceDetails { pid, title } );
        
    }
    return game_instances

}


pub fn tasklist() -> String {
    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("tasklist")
            .creation_flags(0x08000000)
            .args(&["/fi", "IMAGENAME eq D2R.exe", "/v", "/FO", "CSV"])
            .output()
            .expect("failed to execute process")
    } else {
        std::process::Command::new("sh")
            .arg("-c")
            .arg("echo Todo!")
            .output()
            .expect("failed to execute process")
    };
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn get_d2r_instances() -> String {
    let task_list: String = tasklist();
    let mut rdr = csv::Reader::from_reader(task_list.as_bytes());
    let mut d2r_list: Vec<String> = vec![];
    for result in rdr.records() {
        let record = match result {
            Ok(it) => it,
            Err(_) => todo!(),
        };
        
        let pid = record[1].parse::<u32>().unwrap();
        let title = record[8].parse::<String>().unwrap();
        d2r_list.push(format!("{}: '{}'", pid, title));
        
    }
    if d2r_list.len() > 0 {
        d2r_list.insert(0, String::from("D2R Instances currently running:"));
        return d2r_list.join("\n");
    }
    return String::new()
}

