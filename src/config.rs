// Copyright 2018 superfunc, see license.txt for usage details.
//
// Module containing config functionality, including fetching the
// contents of the config file, as well as sub-config info.

use std::fs;

use crate::types::{IndividualBenchInfo, TopLevelBenchInfo};

pub fn get_config_root_dir() -> std::path::PathBuf {
    match dirs::config_dir() {
        Some(root_dir) => {
            let config_dir = root_dir.join(std::path::Path::new("bb/"));
            println!("Using config directory at {:?}", config_dir);
            config_dir
        }
        None => {
            println!("Failed to create config directory");
            std::process::exit(1);
        }
    }
}

pub fn get_individual_config_file(name: &str) -> std::path::PathBuf {
    get_config_root_dir().join(name.to_string()).join("info.json")
}

pub fn get_top_level_config_file() -> std::path::PathBuf {
    get_config_root_dir().join("top.json")
}

pub fn ensure_dependencies_available() {
    if !crate::git::is_available() {
        println!("Failed to find dependency \"git\"");
        std::process::exit(1);
    }
}

pub fn ensure_initialized() {
    // Create our directory structure if needed
    let dir = get_config_root_dir().to_owned();
    if !dir.exists() {
        let config_msg = format!("There is no config directory for bb, can I create one at {}?", &dir.to_string_lossy());
        if dialoguer::Confirmation::new().with_text(&config_msg).interact().is_err() {
            println!("Ok, exiting simulation.");
            return;
        } else if std::fs::create_dir(&dir).is_err() {
            println!("Failed to create new directory at {:?}", dir);
            return;
        }
    }

    // Create our top level config files if needed
    let config_file = get_top_level_config_file();
    if !config_file.exists() {
        let empty_bench_info = TopLevelBenchInfo::new();
        if std::fs::File::create(&config_file).is_err() {
            println!("Failed to create config file");
            return;
        }

        println!("Writing default config file at {:?}", config_file);
        fs::write(&config_file, serde_json::to_string_pretty(&empty_bench_info).unwrap()).unwrap();
    }
}

pub fn read_top_level_config() -> TopLevelBenchInfo {
    let config_file = get_top_level_config_file();
    if !config_file.is_file() && std::fs::File::create(&config_file).is_err() {
        println!("Failed to create config file");
        std::process::exit(1);
    }

    let contents = std::fs::read(&config_file);
    if contents.is_err() {
        println!("Failed to read config file contents");
        std::process::exit(1);
    }

    type ConfigParseResult = Result<TopLevelBenchInfo, serde_json::Error>;
    let utf8_contents = String::from_utf8_lossy(&contents.unwrap()).to_string();
    let benches: ConfigParseResult = serde_json::from_str(&utf8_contents);
    if benches.is_err() {
        println!("Failed to parse json from config");
        std::process::exit(1);
    }

    benches.unwrap()
}

pub fn read_individual_config(name: &str) -> IndividualBenchInfo {
    let config_file = get_individual_config_file(name);
    if !config_file.is_file() {
        println!("Unable to find expected config file: {:?}", config_file);
        std::process::exit(1);
    }

    let contents = std::fs::read(&config_file);
    if contents.is_err() {
        println!("Failed to read config file contents");
    }

    type ConfigParseResult = Result<IndividualBenchInfo, serde_json::Error>;
    let utf8_contents = String::from_utf8_lossy(&contents.unwrap()).to_string();
    let benches: ConfigParseResult = serde_json::from_str(&utf8_contents);
    if benches.is_err() {
        println!("Failed to parse json from config");
        std::process::exit(1);
    }

    benches.unwrap()
}
