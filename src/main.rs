#![feature(concat_idents)]

extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use clap::{App, Arg, SubCommand};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize, Serialize)]
struct BenchResult {
    name: String,
    iterations: i64,
    real_time: f64,
    cpu_time: f64,
    time_unit: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct BenchContextInfo {
    level: i64,
    size: i64,
    num_sharing: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct BenchContext {
    date: String,
    executable: String,
    num_cpus: i64,
    mhz_per_cpu: i64,
    cpu_scaling_enabled: bool,
    caches: Vec<BenchContextInfo>,
    library_build_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct IndividualBenchInfo {
    context: Option<BenchContext>,
    benchmarks: Vec<BenchResult>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BenchHeader {
    // TODO: Make this a Path, not a string
    root: String,
    description: String,
}

type BenchId = String;
type TopLevelBenchInfo = HashMap<BenchId, BenchHeader>;

fn print_banner() { 
    println!("--------------------------------------------------------");
}

macro_rules! config_root {
    () => (env::home_dir().unwrap().join(Path::new(".config/")).join(Path::new("bb/")).as_path())
}

fn read_top_level_config() -> TopLevelBenchInfo {
    let config_file = config_root!().join(Path::new("top.json"));
    if !config_file.is_file() {
        fs::File::create(config_file);
        return TopLevelBenchInfo::new();
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
        let benches: Result<IndividualBenchInfo, serde_json::Error> = serde_json::from_str(&raw_contents);
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
        println!("\nName: {:?}\nDescription: {:?}\nLocation: {:?}", 
                 id, info.description, info.root);
        print_banner();
    }
}

fn handle_new(name: &str) -> () {
    let mut benches = read_top_level_config();
    match benches.get(name) {
        Some(header) => {
            println!("Name {:?} already exists in benchmarks.", name);
        }
        None => {
            // Author skeleton info.json file
            let individual = config_root!().join(name).join("info.json");
            fs::create_dir(individual.parent().unwrap());
            fs::File::create(&individual);
            let blank_individual_config = IndividualBenchInfo {context: None, benchmarks: vec!()};
            fs::write(&individual, serde_json::to_string_pretty(&blank_individual_config).unwrap());

            // Update top level json file
            let top_level = config_root!().join(Path::new("top.json"));
            benches.insert(name.to_string(), BenchHeader{root:path.to_str().unwrap().to_string(), 
                                                         description:"".to_string()});
            fs::write(&top_level, serde_json::to_string_pretty(&benches).unwrap());
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
    // GET command from name
    // let command = ;
    //let output = Command::new(command)
    //    .arg("--benchmark_format=json")
    //    .output()
    //    .unwrap();

    //// TODO: Handle changed context's (running benchmarks from different machines)
    //let raw: String = String::from_utf8_lossy(&output.stdout).to_string();
    //let benches = read_top_level_config();
    ////let mut local_info = read_individual_config(&benches.get(name).unwrap().root);

    //// TODO: Append to actual results
    //fs::write(&benches.get(name).unwrap().root, raw);
}

macro_rules! individual_handle_fn {
    ($name: ident, $matches: ident) => {
        if let Some(v) = ($matches).subcommand_matches(stringify!($name)) {
            match v.value_of("name") {
                Some(n) => concat_idents!(handle_, $name)(n),
                None => panic!("No registered handler function."),
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

macro_rules! individual_command {
    ($name: expr, $desc: expr) => {
        SubCommand::with_name(stringify!($name))
            .about($desc)
            .arg(Arg::with_name("name").value_name("name").index(1))
    };
}

macro_rules! top_level_command {
    ($name: expr, $desc: expr) => {
        SubCommand::with_name(stringify!($name)).about($desc)
    };
}

fn main() {
    let matches = App::new("Benchmarking utility program")
        .version("0")
        .author("superfunc")
        .subcommand(top_level_command!(list, "List available benchmarks"))
        .subcommand(individual_command!(info,"Information on an individual benchmark"))
        .subcommand(individual_command!(new, "Create a new benchmark"))
        .subcommand(individual_command!(report, "Generate an html report for a benchmark"))
        .subcommand(individual_command!(run, "Run another iteration of a benchmark"))
        .get_matches();

    top_level_handle_fn!(list, matches);
    individual_handle_fn!(info, matches);
    individual_handle_fn!(new, matches);
    individual_handle_fn!(run, matches);
}
