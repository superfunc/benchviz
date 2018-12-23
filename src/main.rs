#![feature(concat_idents)]
#![feature(nll)]

mod types;
use self::types::{TopLevelBenchInfo, IndividualBenchInfo, BenchHeader};

use clap::clap_app;
use std::path::Path;
use std::{fs, process};

// Terminal information display functions
fn print_banner() {
    println!("--------------------------------------------------------");
}

fn print_user_error(msg: &str) {
    println!("ERROR: {:}.\nSee usage with \"benchviz -h\"", msg);
}

macro_rules! config_root {
    () => {
        dirs::home_dir()
            .unwrap()
            .join(dirs::config_dir().unwrap())
            .join(Path::new("bb/"))
            .as_path()
    };
}

fn ensure_initialized() {
    let dir = config_root!().to_owned();
    if dir.exists() {
        return;
    }

    if !dialoguer::Confirmation::new().with_text(
        &format!(
            "There is no config directory for bb, \
             can I create one at {}?", 
        &dir.to_string_lossy())).interact().unwrap() 
    {
        println!("Ok, exiting simulation.");
    } else {
        std::fs::create_dir(dir); 
    }
}

fn read_top_level_config() -> TopLevelBenchInfo {
    let config_file = config_root!().join(Path::new("top.json"));
    if !config_file.is_file() {
        match fs::File::create(config_file) {
            Err(_) => panic!("Failed to create config file"),
            _ => return TopLevelBenchInfo::new(),
        }
    }

    let raw_contents = String::from_utf8_lossy(&fs::read(config_file).unwrap()).to_string();
    let benches: Result<TopLevelBenchInfo, serde_json::Error> = serde_json::from_str(&raw_contents);
    match benches.ok() {
        Some(v) => return v,
        None => return TopLevelBenchInfo::new(),
    }
}

fn read_individual_config(name: &str, header: &BenchHeader) -> IndividualBenchInfo {
    let config_file = config_root!().join(Path::new(name)).join("info.json");
    if config_file.is_file() {
        let raw_contents = String::from_utf8_lossy(&fs::read(config_file).unwrap()).to_string();
        let benches: Result<IndividualBenchInfo, serde_json::Error> =
            serde_json::from_str(&raw_contents);
        return benches.unwrap();
    } else {
        panic!("Unable to find expected config file: {:?}", config_file);
    }
}

fn handle_list() -> () {
    let benches = read_top_level_config();
    print_banner();
    println!("{} benchmarks found: ", benches.len());
    for (id, info) in benches {
        println!(
            "\nName: {:?}\nDescription: {:?}\nLocation: {:?}",
            id, info.description, info.root
        );
        print_banner();
    }
}

fn handle_new() -> () {
    let name: String = dialoguer::Input::new()
        .with_prompt("Enter a name for the benchmark")
        .interact()
        .unwrap();
    let loc: String = dialoguer::Input::new()
        .with_prompt("Enter a source location")
        .interact()
        .unwrap();
    let desc: String = dialoguer::Input::new()
        .with_prompt("Describe this benchmark")
        .interact()
        .unwrap();

    let mut benches = read_top_level_config();
    match benches.get(&name) {
        Some(header) => {
            println!("Name {:?} already exists in benchmarks.", name);
        }
        None => {
            // Author skeleton info.json file
            let individual = config_root!().join(&name).join("info.json");
            fs::create_dir(individual.parent().unwrap()).unwrap();
            fs::File::create(&individual).unwrap();
            let blank_individual_config = IndividualBenchInfo {
                context: None,
                benchmarks: vec![],
            };
            fs::write(
                &individual,
                serde_json::to_string_pretty(&blank_individual_config).unwrap(),
            )
            .unwrap();

            // Update top level json file
            let top_level = config_root!().join(Path::new("top.json"));

            benches.insert(
                name.to_string(),
                BenchHeader {
                    root: loc.to_string(),
                    description: desc.to_string(),
                },
            );
            fs::write(&top_level, serde_json::to_string_pretty(&benches).unwrap()).unwrap();
        }
    }
}

fn handle_info(name: &str) -> () {
    let benches = read_top_level_config();
    match benches.get(name) {
        Some(header) => {
            let info = read_individual_config(name, &header);
            println!("# of runs: {:?}", info.benchmarks.len());
            println!("{:?}", info.context);
        }
        None => println!("Name {:?} not found in benches.", name),
    }
}

fn handle_run(name: &str) -> () {
    let benches = read_top_level_config();
    match benches.get(name) {
        Some(header) => {
            let mut info = read_individual_config(name, &header);
            let exe = info.context.as_ref().unwrap().executable.to_string();
            let output = process::Command::new(&exe)
                .arg("--benchmark_format=json")
                .output()
                .unwrap();

            let raw: String = String::from_utf8_lossy(&output.stdout).to_string();
            let mut new_benches: IndividualBenchInfo = serde_json::from_str(&raw).unwrap();
            info.benchmarks.append(&mut new_benches.benchmarks);
            let path = config_root!().join(name).join("info.json");
            fs::write(&path, serde_json::to_string_pretty(&info).unwrap()).unwrap();
        }
        None => println!("Name {:?} not found in benches.", name),
    }
}

macro_rules! individual_handle_fn {
    ($name: ident, $matches: ident) => {
        if let Some(v) = ($matches).subcommand_matches(stringify!($name)) {
            match v.value_of("name") {
                Some(n) => concat_idents!(handle_, $name)(n),
                None => {
                    print_user_error(
                        "This command requires a name \
                         argument denoting the benchmark \
                         it corresponds to.",
                    );
                    process::exit(1);
                }
            }
        }
    };
}

macro_rules! top_level_handle_fn {
    ($name: ident, $matches: ident) => {
        if let Some(_) = ($matches).subcommand_matches(stringify!($name)) {
            concat_idents!(handle_, $name)()
        }
    };
}

fn main() {
    let matches = clap_app!(myapp =>
       (version: "1.0")
       (author: "superfunc <superfunc@users.noreply.github.com>")
       (about: "A utility for managing Google benchmark.")
       (@subcommand list =>
          (about: "List available benchmarks"))
       (@subcommand info =>
          (about: "Information on an individual benchmark")
          (@arg name: "Name of benchmark"))
       (@subcommand new =>
          (about: "Create a new benchmark"))
       (@subcommand report =>
          (about: "Generate an visual report for a benchmark.")
          (@arg name: "Name of benchmark"))
       (@subcommand run =>
          (about: "Run another iteration of a benchmark.")
          (@arg name: "Name of benchmark")))
    .get_matches();

    ensure_initialized();

    top_level_handle_fn!(list, matches);
    top_level_handle_fn!(new, matches);
    individual_handle_fn!(info, matches);
    individual_handle_fn!(run, matches);
}
