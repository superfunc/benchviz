// Copyright 2018 superfunc, see license.txt for usage details.
//
// Module containing io functionality for printing info to users
// in the CLI environment

use std::fs;
use std::process;

use plotlib::line::{Line, Style};
use plotlib::page;
use plotlib::style::Line as OtherLine;

fn open_svg(file: &str) {
    if cfg!(windows) {
        process::Command::new("start").arg(&file).spawn().unwrap();
    } else if cfg!(unix) {
        process::Command::new("open").arg(&file).spawn().unwrap();
    } else {
        panic!("Running on an unsupported operating system, sorry!");
    }
}

fn git_is_available() -> bool {
    if let Ok(_) = process::Command::new("git").output() {
        return true;
    }

    return false;
}

fn get_git_diff(source_root: &str, hash1: &str, hash2: &str) -> String {
    if !git_is_available() {
        return "".to_string();
    }

    match std::env::current_dir() {
        Ok(curr) => {
            std::env::set_current_dir(&source_root).unwrap();
            let output = process::Command::new("git")
                .arg("diff")
                .arg("--color=always")
                .arg(&hash1)
                .arg(&hash2)
                .output()
                .unwrap();
            let raw: String = String::from_utf8_lossy(&output.stdout).to_string();
            std::env::set_current_dir(&curr).unwrap();
            raw.to_string()
        }
        _ => "".to_string(),
    }
}

fn get_git_hash(source_root: &str) -> String {
    if !git_is_available() {
        return "".to_string();
    }

    match std::env::current_dir() {
        Ok(curr) => {
            std::env::set_current_dir(&source_root).unwrap();
            let output = process::Command::new("git")
                .arg("rev-parse")
                .arg("HEAD")
                .output()
                .unwrap();
            let raw: String = String::from_utf8_lossy(&output.stdout).to_string();
            std::env::set_current_dir(&curr).unwrap();
            raw.trim().to_string()
        }
        _ => "".to_string(),
    }
}

fn print_banner() {
    println!(
        "-------------------------------------------------------------\
         -------------------------"
    );
}

fn lookup_benchmark(name: &str) -> crate::types::BenchmarkQuery {
    let benches = crate::config::read_top_level_config();
    match benches.get(name) {
        Some(header) => Some((header.clone(), crate::config::read_individual_config(name))),
        None => {
            println!("Name {:?} not found in benches.", name);
            None
        }
    }
}

pub fn print_comparison(name: &str, run_id_1: usize, run_id_2: usize) {
    print_banner();
    if let Some((header, info)) = lookup_benchmark(name) {
        let num_runs = info.commentary.len();
        if run_id_1 >= num_runs {
            println!(
                "Invalid run id specified ({}), only {} runs recorded",
                run_id_1, num_runs
            );
            return;
        }

        if run_id_2 >= num_runs {
            println!(
                "Invalid run id specified ({}), only {} runs recorded",
                run_id_2, num_runs
            );
            return;
        }

        let bench_results_1 = &info.benchmarks[run_id_1];
        let bench_results_2 = &info.benchmarks[run_id_2];

        use colored::*;

        let diff_str = |lhs: f64, rhs: f64| {
            let abs_str = (lhs - rhs).abs().to_string();

            if lhs > rhs {
                return ("-".to_string() + &abs_str).green();
            } else if lhs < rhs {
                return ("+".to_string() + &abs_str).red();
            } else {
                return "0".to_string().blue();
            }
        };

        println!("Prelude: ");
        print_banner();

        // TODO: Users could alter their benchmarks to be inconsistent
        // we should probably do something to handle this better.
        println!("Comparing run {} and {} from {}", run_id_1, run_id_2, name);
        println!(
            "Run {} description: {}",
            run_id_1, info.commentary[run_id_1]
        );
        println!(
            "Run {} description: {}",
            run_id_2, info.commentary[run_id_2]
        );
        print_banner();
        println!("Time difference(s): ");
        print_banner();
        for i in 0..bench_results_1.len() {
            println!(
                "{}: {}{}{}{}{}: {}",
                "Name".white(),
                bench_results_1[i].name.italic(),
                " ".to_string().repeat(32 - bench_results_1[i].name.len()),
                "Time Diff(".to_string(),
                bench_results_1[i].time_unit.cyan(),
                ")",
                diff_str(bench_results_1[i].real_time, bench_results_2[i].real_time)
            );
        }

        print_banner();
        println!("Source difference(s): ");
        print_banner();
        let hash1 = &info.source_hashes[run_id_1];
        let hash2 = &info.source_hashes[run_id_2];
        println!("{}", get_git_diff(&header.source_root, &hash1, &hash2));
    }
    print_banner();
}

pub fn print_current_benchmarks() {
    print_banner();
    let benches = crate::config::read_top_level_config();
    for (id, info) in benches {
        println!(
            "\nName: {:?}\nDescription: {:?}\nSource Location: {:?}\
             \nExecutable Location: {:?}",
            id, info.description, info.source_root, info.source_bin
        );
        print_banner();
    }
}

pub fn print_individual_bench_info(name: &str) {
    print_banner();
    if let Some((header, info)) = lookup_benchmark(name) {
        println!("Description: {}", header.description);
        for i in 0..info.commentary.len() {
            println!(
                "Run #{} ({}): {}",
                i,
                info.source_hashes[i].get(..8).unwrap(),
                info.commentary[i]
            );
        }
    }
    print_banner();
}

pub fn plot_individual_benchmark(name: &str) {
    if let Some((_, info)) = lookup_benchmark(name) {
        let mut data: Vec<(f64, f64)> = vec![];
        let mut lines: Vec<Line> = vec![];
        let mut v = plotlib::view::ContinuousView::new();
        let colors = vec!["magenta", "pink", "teal", "turquoise"];
        let mut start = 0;
        let mut color_index = 0;

        for run in info.benchmarks {
            let mut i = 0;
            for results in run {
                data.push((i as f64, results.real_time));
                i = i + 1;
            }

            lines.push(
                Line::new(&data[start..data.len()]).style(
                    Style::new()
                        .colour(colors[color_index % colors.len()])
                        .width(4.2),
                ),
            );

            start += i;
            color_index += 1;
        }

        v = v.y_label("Time(ns)");
        for i in 0..lines.len() {
            v = v.add(&lines[i]);
        }

        page::Page::single(&v).save("/tmp/test.svg").unwrap();
        open_svg("/tmp/test.svg");
    }
}

pub fn run_individual_benchmark(name: &str) {
    let desc: String = dialoguer::Input::new()
        .with_prompt("What has changed since the last run?")
        .interact()
        .unwrap();

    if let Some((header, mut info)) = lookup_benchmark(name) {
        let exe = &header.source_bin;
        let output = process::Command::new(&exe)
            .arg("--benchmark_format=json")
            .output()
            .unwrap();

        let raw: String = String::from_utf8_lossy(&output.stdout).to_string();
        let new_benches: crate::types::BenchRunResult = serde_json::from_str(&raw).unwrap();
        info.benchmarks.push(new_benches.benchmarks);
        info.commentary.push(desc);
        info.source_hashes.push(get_git_hash(&header.source_root));
        let path = crate::config::get_individual_config_file(name);
        fs::write(&path, serde_json::to_string_pretty(&info).unwrap()).unwrap();
    }
}

pub fn create_new_individual_benchmark() {
    let name: String = dialoguer::Input::new()
        .with_prompt("Enter a name for the benchmark")
        .interact()
        .unwrap();
    let src: String = dialoguer::Input::new()
        .with_prompt("Enter a source directory location")
        .interact()
        .unwrap();
    let bin: String = dialoguer::Input::new()
        .with_prompt("Enter an executable path")
        .interact()
        .unwrap();
    let desc: String = dialoguer::Input::new()
        .with_prompt("Describe this benchmark")
        .interact()
        .unwrap();

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
                context: None,
                benchmarks: vec![],
                commentary: vec![],
                source_hashes: vec![],
            };
            fs::write(
                &individual,
                serde_json::to_string_pretty(&blank_individual_config).unwrap(),
            )
            .unwrap();

            // Update top level json file
            let top_level = crate::config::get_top_level_config_file();

            benches.insert(
                name.to_string(),
                crate::types::BenchHeader {
                    source_root: src.to_string(),
                    source_bin: bin.to_string(),
                    description: desc.to_string(),
                },
            );
            fs::write(&top_level, serde_json::to_string_pretty(&benches).unwrap()).unwrap();
        }
    }
}

pub fn remove_benchmark_run() {
    print_banner();
    println!("Current benchmarks (run info command for more info): ");
    let benches = crate::config::read_top_level_config();
    for (name, header) in benches {
        println!("\n{}: {}", name, header.description);
    }

    'namePromptLoop: loop {
        let name: String = dialoguer::Input::new()
            .with_prompt("Which benchmark would you like to remove?")
            .interact()
            .unwrap();

        if let Some((_, mut info)) = lookup_benchmark(&name) {
            'runIdPromptLoop: loop {
                let num_runs = info.benchmarks.len();
                let prompt = format!(
                    "{} has {} runs, which would \
                     you like to remove? (Enter * for all)",
                    &name, num_runs
                );
                let run_id: String = dialoguer::Input::new()
                    .with_prompt(&prompt)
                    .interact()
                    .unwrap();

                if run_id == "*" {
                    info.benchmarks.clear();
                    info.commentary.clear();
                    info.source_hashes.clear();
                    break 'runIdPromptLoop;
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

                            info.benchmarks.remove(val);
                            info.commentary.remove(val);
                            info.source_hashes.remove(val);
                            break 'runIdPromptLoop;
                        }
                        Err(_) => {
                            println!("Unparseable unsigned supplied, try again.");
                            continue 'runIdPromptLoop;
                        }
                    }
                }
            }

            let path = crate::config::get_individual_config_file(&name);
            match serde_json::to_string_pretty(&info) {
                Ok(content) => match fs::write(&path, &content) {
                    Ok(_) => {
                        break 'namePromptLoop;
                    }
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
    }

    print_banner();
}
