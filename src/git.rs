// Copyright 2018 superfunc, see license.txt for usage details.
//
// Module containing io functionality for understanding source
// control info pertaining to our benchmarks.

use std::process;

pub fn is_available() -> bool {
    process::Command::new("git").output().is_ok()
}

pub fn diff(source_root: &str, hash1: &str, hash2: &str) -> String {
    if !is_available() {
        return "".to_string();
    }

    match std::env::current_dir() {
        Ok(curr) => {
            std::env::set_current_dir(&source_root).unwrap();
            let output = process::Command::new("git").arg("diff").arg("--color=always").arg(&hash1).arg(&hash2).output().unwrap();
            let raw: String = String::from_utf8_lossy(&output.stdout).to_string();
            std::env::set_current_dir(&curr).unwrap();
            raw.to_string()
        }
        _ => "".to_string()
    }
}

pub fn hash(source_root: &str) -> String {
    if !is_available() {
        return "".to_string();
    }

    match std::env::current_dir() {
        Ok(curr) => {
            std::env::set_current_dir(&source_root).unwrap();
            let output = process::Command::new("git").arg("rev-parse").arg("HEAD").output().unwrap();
            let raw: String = String::from_utf8_lossy(&output.stdout).to_string();
            std::env::set_current_dir(&curr).unwrap();
            raw.trim().to_string()
        }
        _ => "".to_string()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_available() {
        assert!(crate::git::is_available());
    }

    #[test]
    fn check_hash() {
        let curr = std::env::current_dir().unwrap();
        let hash = crate::git::hash(&curr.to_str().unwrap());
        assert!(hash.len() > 0);
    }
}
