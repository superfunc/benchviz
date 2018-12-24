// Module containing config functionality, including fetching the
// contents of the config file, as well as sub-config info.

use crate::types::{IndividualBenchInfo, TopLevelBenchInfo};

#[macro_export]
macro_rules! root_dir {
    () => {
        dirs::home_dir()
            .unwrap()
            .join(dirs::config_dir().unwrap())
            .join(std::path::Path::new("bb/"))
            .as_path()
    };
}

pub fn get_individual_config_file(name: &str) -> std::path::PathBuf {
    root_dir!().join(name.to_string()).join("info.json")
}

pub fn get_individual_config_dir(name: &str) -> std::path::PathBuf {
    root_dir!().join(name.to_string())
}

pub fn get_top_level_config_file() -> std::path::PathBuf {
    root_dir!().join("top.json")
}

pub fn ensure_initialized() {
    let dir = root_dir!().to_owned();
    if dir.exists() {
        return;
    }

    let config_msg = format!(
        "There is no config directory for bb, can I create one at {}?",
        &dir.to_string_lossy()
    );
    if !dialoguer::Confirmation::new()
        .with_text(&config_msg)
        .interact()
        .unwrap()
    {
        println!("Ok, exiting simulation.");
    } else {
        std::fs::create_dir(dir).unwrap();
    }
}

pub fn read_top_level_config() -> TopLevelBenchInfo {
    let config_file = get_top_level_config_file();
    if !config_file.is_file() {
        match std::fs::File::create(config_file) {
            Err(_) => panic!("Failed to create config file"),
            _ => return TopLevelBenchInfo::new(),
        }
    }

    let raw_contents = String::from_utf8_lossy(&std::fs::read(config_file).unwrap()).to_string();
    let benches: Result<TopLevelBenchInfo, serde_json::Error> = serde_json::from_str(&raw_contents);
    match benches.ok() {
        Some(v) => return v,
        None => return TopLevelBenchInfo::new(),
    }
}

pub fn read_individual_config(name: &str) -> IndividualBenchInfo {
    let config_file = get_individual_config_file(name);
    if config_file.is_file() {
        let raw_contents =
            String::from_utf8_lossy(&std::fs::read(config_file).unwrap()).to_string();
        let benches: Result<IndividualBenchInfo, serde_json::Error> =
            serde_json::from_str(&raw_contents);
        return benches.unwrap();
    } else {
        panic!("Unable to find expected config file: {:?}", config_file);
    }
}
