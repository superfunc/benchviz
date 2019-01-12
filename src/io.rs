// Copyright 2018 superfunc, see license.txt for usage details.
//
// Module containing io functionality for printing info to users
// in the CLI environment

use std::fs;
use std::process;

use prettytable::{cell, row};

fn lookup_benchmark(name: &str) -> crate::types::BenchmarkQuery {
    let benches = crate::config::read_top_level_config();
    match benches.get(name) {
        Some(header) => Some((header.clone(), crate::config::read_individual_config(name))),
        None => {
            use colored::*;
            println!("{}", format!("> Name {:?} not found in benches.", name).red());
            println!("{}", "  Currently available benchmarks".red());
            let benches = crate::config::read_top_level_config();
            for (id, _) in benches {
                let fmt = format!("  > Name: {:?}", id);
                println!("{}", fmt.red());
            }
            None
        }
    }
}

pub fn parse_run_id(name: &str, run_id: &str) -> Option<crate::types::RunId> {
    let info = crate::config::read_individual_config(&name);

    if run_id == "*" {
        return Some(crate::types::RunId::All);
    } else {
        match run_id.parse::<usize>() {
            Ok(val) => {
                let len_benches = info.benchmarks.len();
                let len_comments = info.commentary.len();
                let len_hashes = info.commentary.len();

                if len_benches != len_comments || len_comments != len_hashes {
                    println!("Benchmarks file in inconsistent state, perhaps it was hand edited?");
                    std::process::exit(1);
                }

                if val >= len_benches {
                    println!("Invalid run id specified, try again");
                }

                return Some(crate::types::RunId::Index(val));
            }
            Err(_) => {
                println!("Unparseable unsigned supplied, try again.");
            }
        }
    }

    None
}

pub fn prompt_benchmark_name() -> String {
    let prompt = "Which benchmark?";
    loop {
        let name: String = dialoguer::Input::new().with_prompt(&prompt).interact().unwrap();

        if let Some((_, _)) = lookup_benchmark(&name) {
            return name;
        } else {
            // TODO: Refactor all this printing stuff
            use colored::*;
            println!("{}", "Name not found in benchmarks".red());
            println!("{}", "Currently available benchmarks".red());
            let benches = crate::config::read_top_level_config();
            for (id, _) in benches {
                let fmt = format!("> Name: {:?}", id);
                println!("{}", fmt.red());
            }
        }
    }
}

fn prompt_run_id(name: &str) -> crate::types::RunId {
    let info = crate::config::read_individual_config(&name);

    loop {
        let num_runs = info.benchmarks.len();
        let prompt = format!(
            "{} has {} runs, which would \
             you like to remove? (Enter * for all)",
            &name, num_runs
        );
        for i in 0..num_runs {
            println!(" > Run #{}: {}", i, info.commentary[i]);
        }

        let run_id: String = dialoguer::Input::new().with_prompt(&prompt).interact().unwrap();

        if let Some(parsed_run_id) = parse_run_id(&name, &run_id) {
            return parsed_run_id;
        }
    }
}

pub fn print_comparison(name: &str, run_id_1_wrapped: crate::types::RunId, run_id_2_wrapped: crate::types::RunId) {
    if let (Some((header, info)), crate::types::RunId::Index(run_id_1), crate::types::RunId::Index(run_id_2)) =
        (lookup_benchmark(name), run_id_1_wrapped, run_id_2_wrapped)
    {
        let num_runs = info.commentary.len();
        if num_runs == 0 {
            println!("No runs are currently recorded!");
            return;
        }

        if run_id_1 >= num_runs {
            println!("Invalid run id specified ({}), only {} runs recorded", run_id_1, num_runs);
            return;
        }

        if run_id_2 >= num_runs {
            println!("Invalid run id specified ({}), only {} runs recorded", run_id_2, num_runs);
            return;
        }

        let bench_results_1 = &info.benchmarks[run_id_1];
        let bench_results_2 = &info.benchmarks[run_id_2];
        let mut output = prettytable::Table::new();
        output.set_titles(row!["Name", "LHS Time", "RHS Time", "Absolute Diff", "% Diff", "X Speedup"]);

        for result in bench_results_1.iter().zip(bench_results_2.iter()) {
            let (lhs, rhs) = &result;

            let name = &lhs.name;
            let lhs_time = lhs.real_time;
            let rhs_time = rhs.real_time;
            let abs_diff = rhs_time - lhs_time;
            let percent_diff = 100.0 * (rhs_time - lhs_time) / rhs_time;
            let improvement = lhs_time / rhs_time;
            output.add_row(row![
                format!("{}", name),
                format!("{:.3}", lhs_time),
                format!("{:.3}", rhs_time),
                format!("{:.3}", abs_diff),
                format!("{:.3}", percent_diff),
                format!("{:.3}", improvement)
            ]);
        }

        println!("{}", output);
    }
}

pub fn print_current_benchmarks() {
    let benches = crate::config::read_top_level_config();
    for (id, info) in benches {
        println!(
            "> Name: {:?}\n  Description: {:?}\n  Source Location: {:?}\
             \n  Executable Location: {:?}",
            id, info.description, info.source_root, info.source_bin
        );
    }
}

pub fn print_individual_bench_info(name: &str) {
    if let Some((header, info)) = lookup_benchmark(name) {
        println!("> Name: {}", name);
        println!("  Description: {}", header.description);
        println!("  Source Location: {}", header.source_root);
        println!("  Executable Location: {}", header.source_bin);
        println!("  Previous run information: ");
        for i in 0..info.commentary.len() {
            println!("  :: Run #{} (git:{}): {}", i, info.source_hashes[i].get(..8).unwrap(), info.commentary[i]);
        }
    }
}

#[allow(dead_code)]
pub fn print_comparison_with_prompt() {
    unimplemented!();
}

pub fn run_individual_benchmark(name: &str) {
    let desc: String = dialoguer::Input::new().with_prompt("What has changed since the last run?").interact().unwrap();

    if let Some((header, mut info)) = lookup_benchmark(name) {
        let exe = &header.source_bin;
        let output = process::Command::new(&exe).arg("--benchmark_format=json").output().unwrap();

        let raw: String = String::from_utf8_lossy(&output.stdout).to_string();
        let new_benches: crate::types::BenchRunResult = serde_json::from_str(&raw).unwrap();
        info.benchmarks.push(new_benches.benchmarks);
        info.commentary.push(desc);
        info.source_hashes.push(crate::git::hash(&header.source_root));
        let path = crate::config::get_individual_config_file(name);
        fs::write(&path, serde_json::to_string_pretty(&info).unwrap()).unwrap();
    }
}

pub fn create_new_individual_benchmark() {
    let name: String = dialoguer::Input::new().with_prompt("Enter a name for the benchmark").interact().unwrap();
    let src: String = dialoguer::Input::new().with_prompt("Enter a source directory location").interact().unwrap();
    let bin: String = dialoguer::Input::new().with_prompt("Enter an executable path").interact().unwrap();
    let desc: String = dialoguer::Input::new().with_prompt("Describe this benchmark").interact().unwrap();

    let mut benches = crate::config::read_top_level_config();
    match benches.get(&name) {
        Some(_) => {
            println!("Name {:?} already exists in benchmarks.", name);
        }
        None => {
            // Author skeleton info.json file
            let individual = crate::config::get_individual_config_file(&name);
            fs::create_dir(individual.parent().unwrap()).unwrap();
            fs::File::create(&individual).unwrap();
            let blank_individual_config = crate::types::IndividualBenchInfo {
                context:       None,
                benchmarks:    vec![],
                commentary:    vec![],
                source_hashes: vec![]
            };
            fs::write(&individual, serde_json::to_string_pretty(&blank_individual_config).unwrap()).unwrap();

            // Update top level json file
            let top_level = crate::config::get_top_level_config_file();

            benches.insert(
                name.to_string(),
                crate::types::BenchHeader {
                    source_root: src.to_string(),
                    source_bin:  bin.to_string(),
                    description: desc.to_string()
                }
            );
            fs::write(&top_level, serde_json::to_string_pretty(&benches).unwrap()).unwrap();
        }
    }
}

pub fn remove_benchmark_run(name: &str, run_id: &crate::types::RunId) {
    let mut info = crate::config::read_individual_config(&name);
    match run_id {
        crate::types::RunId::All => {
            info.benchmarks.clear();
            info.commentary.clear();
            info.source_hashes.clear();
        }
        crate::types::RunId::Index(val) => {
            info.benchmarks.remove(*val);
            info.commentary.remove(*val);
            info.source_hashes.remove(*val);
        }
    }

    let path = crate::config::get_individual_config_file(&name);
    match serde_json::to_string_pretty(&info) {
        Ok(content) => match fs::write(&path, &content) {
            Ok(_) => {}
            Err(_) => {
                println!("Failed to write results to config file.");
            }
        },
        Err(_) => {
            println!("Failed to write results back to json.");
            std::process::exit(1);
        }
    }
}

pub fn remove_benchmark_run_with_prompt() {
    println!("Current benchmarks (run info command for more info): ");
    let benches = crate::config::read_top_level_config();
    for (name, header) in benches {
        println!(" > {}: {}", name, header.description);
    }

    let name = prompt_benchmark_name();
    let run_id = prompt_run_id(&name);
    remove_benchmark_run(&name, &run_id);
}
