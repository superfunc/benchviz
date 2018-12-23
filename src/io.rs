// Module containing io functionality for printing info to users
// in the CLI environment

use crate::types::{IndividualBenchInfo, TopLevelBenchInfo, BenchHeader};
use crate::config::{read_individual_config, read_top_level_config,
                    get_individual_config_file, get_top_level_config_file};

use std::fs;
use std::process;

pub fn print_banner() {
    println!("--------------------------------------------------------");
}

pub fn print_user_error(msg: &str) {
    println!("ERROR: {:}.\nSee usage with \"benchviz -h\"", msg);
}

pub fn print_current_benchmarks() {
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

pub fn print_individual_bench_info(name: &str) {
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

pub fn run_individual_benchmark(name: &str) {
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
            let path = get_individual_config_file(name); 
            fs::write(&path, serde_json::to_string_pretty(&info).unwrap()).unwrap();
        }
        None => println!("Name {:?} not found in benches.", name),
    }
}

pub fn create_new_individual_benchmark() {
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
            let individual = get_individual_config_file(&name);
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
            let top_level = get_top_level_config_file(); 

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
