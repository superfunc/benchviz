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
    time_unit: String
}

#[derive(Debug, Deserialize, Serialize)]
struct BenchContextInfo {
    level: i64,
    size: i64,
    num_sharing: i64
}

#[derive(Debug, Deserialize, Serialize)]
struct BenchContext {
    date: String,
    executable: String,
    num_cpus: i64,
    mhz_per_cpu: i64,
    cpu_scaling_enabled: bool,
    caches: Vec<BenchContextInfo>,
    library_build_type: String
}

#[derive(Debug, Deserialize, Serialize)]
struct IndividualBenchInfo {
    context: BenchContext,
    benchmarks: Vec<BenchResult>
}

#[derive(Debug, Deserialize, Serialize)]
struct BenchHeader {
    // TODO: Make this a Path, not a string
    root: String,
    description: String
}

type BenchId = String;
type BenchInfo = HashMap<BenchId, Vec<BenchResult>>;

// XXX: Figure out naming here
type TopLevelBenchInfo = HashMap<BenchId, BenchHeader>;

fn read_config() -> TopLevelBenchInfo {
    let config_file = env::current_dir().unwrap().join(Path::new("bb.json"));
    if !config_file.is_file() {
        return TopLevelBenchInfo::new();
    }

    let raw_contents = String::from_utf8_lossy(&fs::read(config_file).unwrap()).to_string();
    let benches: Result<TopLevelBenchInfo, serde_json::Error> = serde_json::from_str(&raw_contents);
    match benches.ok() {
        Some(v) => return v,
        None => return TopLevelBenchInfo::new()
    }
}

fn handle_list() -> () {
    let benches = read_config();
    for (id, info) in benches {
        println!("Name: {:?}\nDescription: {:?}\nLocation{:?}", id, info.description, info.root);
    }
}

fn handle_info(name: &str) -> () {
    let benches = read_config(); 
    match benches.get(name) {
        Some(v) => {
            let path = Path::new(&v.root).parent().unwrap().join(Path::new("bb.json"));
            println!("Path checking: {:?}", path);
            if path.is_file() {
                let raw_contents = String::from_utf8_lossy(&fs::read(path).unwrap()).to_string();
                let benches: Result<IndividualBenchInfo, serde_json::Error> = serde_json::from_str(&raw_contents);
                match benches.ok() {
                    Some(bi) => println!("# of runs: {:?}", bi.benchmarks.len()),
                    None => println!("uhhh")
                }
            }
        }
        None => println!("Name not found in benches."),
    }
}

fn handle_run(command: &str, name: &str) -> () {
    //let output = Command::new(command)
    //    .arg("--benchmark_format=json")
    //    .output()
    //    .unwrap();

    //let raw: String = String::from_utf8_lossy(&output.stdout).to_string();

    let mut benches = read_config(); 

    benches.insert(name.to_string(), BenchHeader{root:command.to_string(), description: String::new()});
    let config_file = env::current_dir().unwrap().join(Path::new("bb.json"));
    let json_contents = serde_json::to_string_pretty(&benches);
    fs::write(config_file, json_contents.unwrap());
}

fn main() {
    let matches = App::new("Benchmarking utility program")
        .version("0.0")
        .author("superfunc")
        .subcommand(SubCommand::with_name("list").about("list benchmarks"))
        .subcommand(
            SubCommand::with_name("info").about("information on an individual benchmark").arg(
                Arg::with_name("name")
                    .long("name")
                    .value_name("name")
                    .help("XXX")
                    .takes_value(true),
            ),
        )
        .subcommand(SubCommand::with_name("new").about("create a new benchmark"))
        .subcommand(SubCommand::with_name("report").about("Create a report for a benchmark"))
        .subcommand(
            SubCommand::with_name("run")
                .about("Run benchmark")
                .arg(
                    Arg::with_name("name")
                        .long("name")
                        .value_name("name")
                        .help("XXX")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("binPath")
                        .long("binPath")
                        .value_name("binPath")
                        .help("benchmark to run")
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(v) = matches.subcommand_matches("list") {
        handle_list();
    }

    if let Some(v) = matches.subcommand_matches("info") {
        match v.value_of("name") {
            Some(n) => handle_info(n),
            None => panic!("ff"),
        }
    }

    if let Some(v) = matches.subcommand_matches("run") {
        match (v.value_of("binPath"), v.value_of("name")) {
            (Some(b), Some(n)) => handle_run(b, n),
            (_, _) => panic!("ff"),
        }
    }
}
