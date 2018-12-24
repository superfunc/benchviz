// Module containing io functionality for printing info to users
// in the CLI environment

use std::fs;
use std::process;

use plotlib::line::{Line, Style};
use plotlib::page;
use plotlib::style::Line as OtherLine;

use crate::config::{
    get_individual_config_file, get_top_level_config_file, read_individual_config,
    read_top_level_config,
};
use crate::types::{BenchHeader, BenchRunResult, IndividualBenchInfo};

pub fn open_svg(file: &str) {
    if cfg!(windows) {
        process::Command::new("start").arg(&file).spawn().unwrap();
    } else if cfg!(unix) {
        process::Command::new("open").arg(&file).spawn().unwrap();
    } else {
        panic!("Running on an unsupported operating system, sorry!");
    }
}

pub fn print_banner() {
    println!("--------------------------------------------------------");
}

pub fn print_comparison(name: &str, run_id_1: usize, run_id_2: usize) {
    let benches = read_top_level_config();
    match benches.get(name) {
        Some(_) => {
            let info = read_individual_config(name);
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

            // TODO: Color this output red/green for good/bad
            let diff_str = |lhs: f64, rhs: f64| {
                use colored::*;

                let abs_str = (lhs - rhs).abs().to_string();

                if lhs > rhs {
                    return ("+".to_string() + &abs_str).green();
                } else if lhs < rhs {
                    return ("-".to_string() + &abs_str).red();
                } else {
                    return "=".to_string().blue();
                }
            };

            // TODO: Users could alter their benchmarks to be inconsistent
            // we should probably do something to handle this better.
            // TODO: Check the actual time units as they may vary.
            for i in 0..num_runs {
                println!(
                    "Name: {}, Real time (ns): {}",
                    bench_results_1[i].name,
                    diff_str(bench_results_1[i].real_time, bench_results_2[i].real_time)
                );
            }
        }
        None => println!("Name {:?} not found in benches.", name),
    }
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
    print_banner();
    let benches = read_top_level_config();
    match benches.get(name) {
        Some(header) => {
            let info = read_individual_config(name);
            println!("Description: {}", header.description);
            for i in 0..info.commentary.len() {
                println!("Run #{}: {}", i, info.commentary[i]);
            }
        }
        None => println!("Name {:?} not found in benches.", name),
    }
    print_banner();
}

pub fn plot_individual_benchmark(name: &str) {
    let benches = read_top_level_config();
    match benches.get(name) {
        Some(_) => {
            let info = read_individual_config(name);
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
        None => println!("Name {:?} not found in benches.", name),
    }
}

pub fn run_individual_benchmark(name: &str) {
    let desc: String = dialoguer::Input::new()
        .with_prompt("What has changed since the last run?")
        .interact()
        .unwrap();

    let benches = read_top_level_config();
    match benches.get(name) {
        Some(_) => {
            let mut info = read_individual_config(name);
            // TODO: Fix this, still very fragile
            let exe = info.context.as_ref().unwrap().executable.to_string();
            let output = process::Command::new(&exe)
                .arg("--benchmark_format=json")
                .output()
                .unwrap();

            let raw: String = String::from_utf8_lossy(&output.stdout).to_string();
            let new_benches: BenchRunResult = serde_json::from_str(&raw).unwrap();
            info.benchmarks.push(new_benches.benchmarks);
            info.commentary.push(desc);
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
        Some(_) => {
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
                commentary: vec![],
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
