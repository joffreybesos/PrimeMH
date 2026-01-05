use serde_json::Error;
use std::os::windows::process::CommandExt;
use std::time::Instant;
use std::{env, fs};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::LOCALISATION;
use crate::settings::Settings;

use super::cache;
use super::jsondata::SeedData;
use super::seeddata::SeedRequest;

pub fn get_seed_data(seed_request: SeedRequest) -> SeedData {
    let localisation = LOCALISATION.lock().unwrap();
    let cached_seed_data_file = cache::cached_file_name(&seed_request.map_seed, &seed_request.difficulty);
    let seed_data_str: String = if Path::new(&cached_seed_data_file).exists() {
        log::info!("Reading cached map data from file {}", &cached_seed_data_file.to_str().unwrap());
        cache::read_cached_file(&cached_seed_data_file)
    } else {
        log::info!(
            "Generating fresh data for seed {} and difficulty {}",
            seed_request.map_seed,
            seed_request.difficulty
        );
        generate_data(seed_request)
    };
    let json: Result<SeedData, Error> = serde_json::from_str(&seed_data_str);
    match json {
        Ok(json) => json,
        Err(_e) => {
            delete_cached_file(&cached_seed_data_file);
            log::debug!("Couldn't get seed data");
            panic!("{}", localisation.get_primemh("error7"))
        }
    }
}

pub fn is_blacha_ok(settings: &Settings) -> Result<bool, String> {
    let localisation = LOCALISATION.lock().unwrap();
    let seed_request = SeedRequest {
        map_seed: 123,
        difficulty: 2,
        d2lodpath: settings.general.d2lodpath.clone(),
        blacha_exe: settings.general.blacha_exe.clone(),
    };

    let d2lodpath = get_path_as_str(&settings.general.d2lodpath);
    
    if !settings.general.d2lodpath.exists() {
        let msg = format!("{}\n{}", localisation.get_primemh("error8"), d2lodpath);
        panic!("{}", msg);
    }
    if !settings.general.d2lodpath.join("d2data.mpq").exists() {
        let msg = format!("{}\n{}", localisation.get_primemh("error8"), d2lodpath);
        panic!("{}", msg);
    }

    let blacha_exe = get_path_as_str(&settings.general.blacha_exe);
    
    if !settings.general.blacha_exe.exists() {
        let msg = format!("{}\n{}", localisation.get_primemh("error10"), blacha_exe);
        panic!("{}", msg);
    }
    let d2lodpath = seed_request.d2lodpath.clone().canonicalize().expect("Failed to get absolute path for d2lodpath");
    
    log::info!(
        "Generating fresh data for seed {} and difficulty {} d2lod: {} blacha: {}",
        seed_request.map_seed,
        seed_request.difficulty,
        seed_request.d2lodpath.to_string_lossy(),
        seed_request.blacha_exe.to_string_lossy()
    );
    let seed_data_str = generate_data(seed_request);

    let json: Result<SeedData, Error> = serde_json::from_str(&seed_data_str);
    match json {
        Ok(_) => Ok(true),
        Err(e ) => {
            let d2log_absolute_path = d2lodpath.to_str().unwrap();
            log::debug!("Error parsing map data, path used: {}", d2log_absolute_path);
            log::debug!("{}", e);
            let forbidden_folders: Vec<&str> = vec!["Desktop", "Dropbox", "Google Drive", "OneDrive"];
            
            for folder in forbidden_folders {
                if d2log_absolute_path.contains(folder) {
                    log::error!("{}\n{}", localisation.get_primemh("error1"), folder);
                }
            }
            panic!("{}", localisation.get_primemh("error2"));
        }
    }
}

fn get_path_as_str(path: &PathBuf) -> String {
    let new_path = if path.is_absolute() {
        PathBuf::from(path)
    } else {
        let current_dir = env::current_dir().expect("Failed to get current directory");
        current_dir.join(path)
    };
    new_path.to_string_lossy().replace("/","\\").replace("\\\\","\\")
}

fn delete_cached_file(cached_seed_data_file: &PathBuf) {
    fs::remove_file(cached_seed_data_file).unwrap();
}

fn is_running_in_wine() -> bool {
    env::var("WINEDEBUG").is_ok() || env::var("WINEPREFIX").is_ok()
}

fn generate_data(seed_request: SeedRequest) -> String {
    let d2lod_absolute_path = seed_request.d2lodpath.canonicalize().expect("Failed to get absolute path for d2lodpath").to_string_lossy().replace("\\\\?\\", "");
    // generate data
    let start = Instant::now();

    // let output = if is_running_in_wine() {
    //     log::info!("Running in wine d2lod_absolute_path: {:?} blacha {:?}", d2lod_absolute_path, seed_request.blacha_exe);
    
    //     Command::new("wine")
    //         .arg(seed_request.blacha_exe)
    //         .arg(d2lod_absolute_path)
    //         .arg("--seed")
    //         .arg(seed_request.map_seed.to_string())
    //         .arg("--difficulty")
    //         .arg(seed_request.difficulty.to_string())
    //         // .arg("--map")
    //         // .arg("1")
    //         .env("WINEPREFIX", "/app/wine_d2")
    //         .env("WINEDEBUG", "-all,fixme-all")
    //         .env("WINEARCH", "win32")
    //         .output()
    //         .unwrap()
    // } else {
    let temp_directory = env::temp_dir();
    let output = Command::new(seed_request.blacha_exe)
        .creation_flags(0x08000000)
        // .arg("/C")
        // .arg("--help")
        .arg(d2lod_absolute_path)
        .arg("--seed")
        .arg(seed_request.map_seed.to_string())
        .arg("--difficulty")
        .arg(seed_request.difficulty.to_string())
        .arg("--json-path")
        .arg(temp_directory.to_str().unwrap())
        .output()
        .unwrap();
    // };
    
    log::info!("Map data generation took {:.3} seconds", (start.elapsed().as_millis() as f64 / 1000.0));

    // save to file
    let cached_seed_data_file = cache::cached_file_name(&seed_request.map_seed, &seed_request.difficulty);
    if seed_request.map_seed == 123 {
        if !cached_seed_data_file.exists() {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();
            log::error!("Could not generate D2 map data, full error:\n{}\n{}", stdout, stderr);
            return String::new();
        }
    }
    return cache::read_cached_file(&cached_seed_data_file);
}



#[cfg(test)]
mod tests {
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use crate::logger::configure_logging;
    use crate::{mapgeneration::{blacha, jsondata::SeedData, seeddata::SeedRequest}};

    
    #[test]
    fn generate_test_seeds() {
        configure_logging();

        let settings = crate::settings::Settings::default();
        let seed_request = SeedRequest {
            map_seed: 123,
            difficulty: 0,
            d2lodpath: settings.general.d2lodpath.clone(),
            blacha_exe: settings.general.blacha_exe.clone(),
        };
        log::info!("Getting seed data");
        let seed_data_json: SeedData = blacha::get_seed_data(seed_request);
            
        
    }
}