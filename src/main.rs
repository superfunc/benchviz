// Copyright 2018 superfunc, see license.txt for usage details.

mod cli;
mod config;
mod git;
mod io;
mod types;

use clap::{clap_app, ArgMatches};

fn main() {
    let matches = clap_app!(benchviz =>
       (version: "1.0")
       (author: "superfunc <superfunc@users.noreply.github.com>")
       (about: "A utility for managing C++ benchmarks.")
       (@subcommand list =>
          (about: "List available benchmarks"))
       (@subcommand new =>
          (about: "Create a new benchmark"))
       (@subcommand info =>
          (about: "Information on an individual benchmark")
          (@arg name: +required "Name of benchmark"))
       (@subcommand run =>
          (about: "Run another iteration of a benchmark.")
          (@arg name: +required "Name of benchmark"))
       (@subcommand remove =>
          (about: "Remove an entire benchmark, or a particular run.")
          (@arg name: +required "Name of benchmark")
          (@arg run_id: +required "Index of the benchmark run (0-indexed)"))
       (@subcommand compare =>
          (about: "Compare two runs from a benchmark")
          (@arg name: +required "Name of benchmark")
          (@arg run_id_1: +required "Index of the first run")
          (@arg run_id_2: +required "Index of the second run")))
    .get_matches();

    config::ensure_dependencies_available();
    config::ensure_initialized();

    cli::handle_global_query("list", &matches, &io::print_current_benchmarks);
    cli::handle_global_query("new", &matches, &io::create_new_individual_benchmark);
    cli::handle_benchmark_query("info", &matches, &io::print_individual_bench_info);
    cli::handle_benchmark_query("run", &matches, &io::run_individual_benchmark);
    cli::handle_run_data_query("remove", &matches, &io::remove_benchmark_run, &io::remove_benchmark_run_with_prompt);
    cli::handle_multi_run_data_query("compare", &matches, &io::print_comparison, &io::print_comparison_with_prompt);
}
