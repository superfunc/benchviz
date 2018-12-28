// Copyright 2018 superfunc, see license.txt for usage details.

mod config;
mod io;
mod types;

use clap::{clap_app, ArgMatches};

// Honestly I'm probably just missing how to get clap to do this
// behavior naturally, but for now we'll just write a little function.
fn parse_bench_id<'a>(matches: &'a ArgMatches, id: &str) -> (Option<&'a str>, Option<&'a str>) {
    match (matches.value_of("name"), matches.value_of(&id)) {
        (Some(name), Some(run_id)) => (Some(name), Some(run_id)),
        (None, None) => (None, None),
        (_, _) => {
            println!(
                "Error: must supply either both <name> and <run_id> or neither. \
                 In the case of neither, a prompt will guide you."
            );
            std::process::exit(1);
        }
    }
}

// Global queries require no benchmark identifier; they speak on the global state of the program
fn handle_global_query(id: &str, matches: &ArgMatches, f: &Fn() -> ()) {
    if matches.subcommand_matches(&id).is_some() {
        f();
    }
}

// Benchmark queries require a valid benchmark identifier; they speak on the specifics for a benchmark.
fn handle_benchmark_query(id: &str, matches: &ArgMatches, f: &Fn(&str) -> ()) {
    if let Some(v) = matches.subcommand_matches(&id) {
        f(v.value_of("name").unwrap());
    }
}

fn handle_run_data_query(id: &str, matches: &ArgMatches, f: &Fn(&str, &types::RunId) -> (), g: &Fn() -> ()) {
    if let Some(v) = matches.subcommand_matches(&id) {
        match parse_bench_id(&v, "run_id") {
            (Some(name), Some(run_id)) => {
                if let Some(parsed_run_id) = io::parse_run_id(&name, &run_id) {
                    f(&name, &parsed_run_id);
                }
            },
            (None, None) => g(),
            (_, _) => unreachable!()
        }
    }
}

fn handle_multi_run_data_query(id: &str, matches: &ArgMatches, f: &Fn(&str, types::RunId, types::RunId) -> (), g: &Fn() -> ()) {
    if let Some(v) = matches.subcommand_matches(&id) {
        match (v.value_of("name"), v.value_of("run_id_1"), v.value_of("run_id_2")) {
            (Some(name), Some(run_id_1), Some(run_id_2)) => {
                match (io::parse_run_id(&name, &run_id_1), io::parse_run_id(&name, &run_id_2)) {
                    (Some(parsed_run_id_1), Some(parsed_run_id_2)) => f(&name, parsed_run_id_1, parsed_run_id_2),
                    (_, _) => {}
                }
            }
            (None, None, None) => g(),
            (_, _, _) => unreachable!()
        }
    }
}

fn main() {
    let matches = clap_app!(benchviz =>
       (version: "1.0")
       (author: "superfunc <superfunc@users.noreply.github.com>")
       (about: "A utility for managing C++ benchmarks.")
       (@subcommand list =>
          (about: "List available benchmarks"))
       (@subcommand new =>
          (about: "Create a new benchmark"))
       (@subcommand plot =>
          (about: "Plot the existing runs of a benchmark.")
          (@arg name: +required "Name of benchmark"))
       (@subcommand info =>
          (about: "Information on an individual benchmark")
          (@arg name: +required "Name of benchmark"))
       (@subcommand remove =>
          (about: "Remove an entire benchmark, or a particular run.")
          (@arg name: "Name of benchmark")
          (@arg run_id: "Index of the benchmark run (0-indexed)"))
       (@subcommand compare =>
          (about: "Compare two runs from a benchmark")
          (@arg name: +required "Name of benchmark")
          (@arg run_id_1: +required "Index of the first run")
          (@arg run_id_2: +required "Index of the second run"))
       (@subcommand run =>
          (about: "Run another iteration of a benchmark.")
          (@arg name: +required "Name of benchmark")))
    .get_matches();

    config::ensure_initialized();

    handle_global_query("list", &matches, &io::print_current_benchmarks);
    handle_global_query("new", &matches, &io::create_new_individual_benchmark);
    handle_benchmark_query("plot", &matches, &io::plot_individual_benchmark);
    handle_benchmark_query("info", &matches, &io::print_individual_bench_info);
    handle_benchmark_query("run", &matches, &io::run_individual_benchmark);
    handle_run_data_query("remove", &matches, &io::remove_benchmark_run, &io::remove_benchmark_run_with_prompt);
    handle_multi_run_data_query("compare", &matches, &io::print_comparison, &io::print_comparison_with_prompt);
}
