// Copyright 2018 superfunc, see license.txt for usage details.
//
// Module containing io functionality for printing info to users
// in the CLI environment

use std::fs;
use std::process;

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

        use colored::*;

        let diff_str = |lhs: f64, rhs: f64| {
            let abs_str = (lhs - rhs).abs().to_string();

            if lhs > rhs {
                ("-".to_string() + &abs_str).green()
            } else if lhs < rhs {
                ("+".to_string() + &abs_str).red()
            } else {
                "0".to_string().blue()
            }
        };

        let sep = "+-----------------------------------------------------------------\
                   --------------------------------------+";

        println!("{}", sep);
        println!("Comparing run {} and {} from {}", run_id_1, run_id_2, name);
        println!("Run {} description: {}", run_id_1, info.commentary[run_id_1]);
        println!("Run {} description: {}", run_id_2, info.commentary[run_id_2]);

        let padding = 3;

        let mut longest_name = 0;
        let mut longest_run_1_time = 0;
        let mut longest_run_2_time = 0;
        let mut longest_time_diff = 0;
        for (i, b) in bench_results_1.iter().enumerate() {
            if b.name.len() > longest_name {
                longest_name = b.name.len();
            }

            let formatted_time = format!("{}", b.real_time);
            if formatted_time.len() > longest_run_1_time {
                longest_run_1_time = formatted_time.len();
            }

            let diff = diff_str(bench_results_1[i].real_time, bench_results_2[i].real_time);
            if diff.len() > longest_time_diff {
                longest_time_diff = diff.len();
            }
        }

        longest_name += padding;

        for b in bench_results_2 {
            let formatted_time = format!("{}", b.real_time);
            if formatted_time.len() > longest_run_2_time {
                longest_run_2_time = formatted_time.len();
            }
        }

        longest_run_1_time += padding;
        longest_run_2_time += padding;
        longest_time_diff += padding;

        let name_label = "Name";
        let run_1_label = format!("Run {} Time", run_id_1);
        let run_2_label = format!("Run {} Time", run_id_2);

        // TODO: Put in units from gbench
        let time_diff_label = "Time Diff(ns)";
        let percent_diff_label = "% Diff";

        println!("{}", sep);

        println!(
            "{}{}{}{}{}{}{}{}{}",
            name_label.white(),
            " ".to_string().repeat(longest_name - name_label.len()),
            run_1_label.white(),
            " ".to_string().repeat(longest_run_1_time - run_1_label.len()),
            run_2_label.white(),
            " ".to_string().repeat(longest_run_2_time - run_2_label.len()),
            time_diff_label.white(),
            " ".to_string().repeat(longest_time_diff - time_diff_label.len()),
            percent_diff_label.white()
        );
        for i in 0..bench_results_1.len() {
            let name_len = bench_results_1[i].name.len();
            let run_1_time = format!("{}", bench_results_1[i].real_time);
            let run_2_time = format!("{}", bench_results_2[i].real_time);
            let time_diff = diff_str(bench_results_1[i].real_time, bench_results_2[i].real_time);
            // TODO: God this code is so bad right now lol
            let mut max = 0.0;
            if bench_results_1[i].real_time > bench_results_2[i].real_time {
                max = bench_results_1[i].real_time;
            } else {
                max = bench_results_2[i].real_time;
            }

            let percent_diff = 100.0 * (bench_results_1[i].real_time - bench_results_2[i].real_time).abs() / max;

            println!(
                "{}{}{}{}{}{}{}{}{}",
                bench_results_1[i].name.italic(),
                " ".to_string().repeat(longest_name - name_len),
                run_1_time,
                " ".to_string().repeat(longest_run_1_time - run_1_time.len()),
                run_2_time,
                " ".to_string().repeat(longest_run_2_time - run_2_time.len()),
                time_diff,
                " ".to_string().repeat(longest_time_diff - time_diff.len()),
                percent_diff
            );
        }
        println!("{}", sep);
        println!("Source difference(s): ");
        let hash1 = &info.source_hashes[run_id_1];
        let hash2 = &info.source_hashes[run_id_2];
        println!("{}", crate::git::diff(&header.source_root, &hash1, &hash2));
        println!("{}", sep);
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
