#![feature(concat_idents)]

mod config;
mod io;
mod types;

use self::config::{ensure_initialized};
use self::io::{print_current_benchmarks, print_individual_bench_info,
               print_user_error,
               create_new_individual_benchmark, run_individual_benchmark};

use clap::clap_app;

fn handle_list() -> () {
    print_current_benchmarks();
}

fn handle_new() -> () {
    create_new_individual_benchmark();
}

fn handle_info(name: &str) -> () {
    print_individual_bench_info(name);
}

fn handle_run(name: &str) -> () {
    run_individual_benchmark(name);
}

macro_rules! individual_handle_fn {
    ($name: ident, $matches: ident) => {
        if let Some(v) = ($matches).subcommand_matches(stringify!($name)) {
            match v.value_of("name") {
                Some(n) => concat_idents!(handle_, $name)(n),
                None => {
                    print_user_error(
                        "This command requires a name \
                         argument denoting the benchmark \
                         it corresponds to.",
                    );
                    std::process::exit(1);
                }
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

fn main() {
    let matches = clap_app!(myapp =>
       (version: "1.0")
       (author: "superfunc <superfunc@users.noreply.github.com>")
       (about: "A utility for managing Google benchmark.")
       (@subcommand list =>
          (about: "List available benchmarks"))
       (@subcommand info =>
          (about: "Information on an individual benchmark")
          (@arg name: "Name of benchmark"))
       (@subcommand new =>
          (about: "Create a new benchmark"))
       (@subcommand report =>
          (about: "Generate an visual report for a benchmark.")
          (@arg name: "Name of benchmark"))
       (@subcommand run =>
          (about: "Run another iteration of a benchmark.")
          (@arg name: "Name of benchmark")))
    .get_matches();

    ensure_initialized();

    top_level_handle_fn!(list, matches);
    top_level_handle_fn!(new, matches);
    individual_handle_fn!(info, matches);
    individual_handle_fn!(run, matches);
}
