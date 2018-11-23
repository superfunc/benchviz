extern crate clap;
extern crate csv;
#[macro_use]
extern crate serde_derive;

use clap::{App, Arg, SubCommand};
use std::env;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize, Serialize)]
struct BenchRecord {
    name: String,
    iterations: i64,
    real_time: f64,
    cpu_time: f64,
    time_unit: String,
    bytes_per_second: Option<i64>,
    items_per_second: Option<i64>,
    error_occurred: Option<String>,
    error_message: Option<String>
}

fn handle_info() -> () {
    let config_file = env::current_dir().unwrap().join(Path::new("bb.json"));
    if !config_file.is_file() {
        println!("Benchmark info file not found!");
        return;
    }

    println!("Benchmark info found!");
}

fn handle_run(command: &str) -> () {
    let output = Command::new(command)
        .arg("--benchmark_format=csv")
        .output()
        .unwrap();

    let raw : String = String::from_utf8_lossy(&output.stdout).to_string();
    let mut csv_reader = csv::Reader::from_reader(raw.as_bytes());
    let mut bench_results : Vec<BenchRecord> = Vec::new();
    for result in csv_reader.deserialize() {
        let record: Result<BenchRecord, csv::Error> = result;
        bench_results.push(record.unwrap());
    }

    println!("{:?}", bench_results);
}

fn main() {
    let matches = App::new("Benchmarking utility program")
        .version("0.0")
        .author("superfunc")
        .subcommand(SubCommand::with_name("info").about("list benchmarks"))
        .subcommand(SubCommand::with_name("new").about("create a new benchmark"))
        .subcommand(SubCommand::with_name("report").about("Create a report for a benchmark"))
        .subcommand(SubCommand::with_name("run")
                    .about("Run benchmark")
                    .arg(Arg::with_name("binPath")
                               .long("binPath")
                               .value_name("binPath")
                               .help("benchmark to run")
                               .takes_value(true)))
        .get_matches();

    if let Some(_) = matches.subcommand_matches("info") {
        handle_info();
    }

    if let Some(v) = matches.subcommand_matches("run") {
        if let Some(bin_path) = v.value_of("binPath") {
            handle_run(bin_path);
        }
    }
}