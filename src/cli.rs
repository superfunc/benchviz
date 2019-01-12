// Copyright 2018 superfunc, see license.txt for usage details.
//
// Module containing io functionality for printing info to users
// in the CLI environment

use clap;

// Honestly I'm probably just missing how to get clap to do this
// behavior naturally, but for now we'll just write a little function.
pub fn parse_bench_id<'a>(matches: &'a clap::ArgMatches, id: &str) -> (Option<&'a str>, Option<&'a str>) {
    match (matches.value_of("name"), matches.value_of(&id)) {
        (Some(name), Some(run_id)) => (Some(name), Some(run_id)),
        (None, None) => (None, None),
        (_, _) => {
            use colored::*;

            println!(
                "{}",
                "Error: must supply either both <name> and <run_id> or neither. \
                 In the case of neither, a prompt will guide you."
                    .red()
            );
            std::process::exit(1);
        }
    }
}

// Global queries require no benchmark identifier; they speak on the global state of the program
pub fn handle_global_query(id: &str, matches: &clap::ArgMatches, f: &Fn() -> ()) {
    if matches.subcommand_matches(&id).is_some() {
        f();
    }
}

// Benchmark queries require a valid benchmark identifier; they speak on the specifics for a benchmark.
pub fn handle_benchmark_query(id: &str, matches: &clap::ArgMatches, f: &Fn(&str) -> ()) {
    if let Some(v) = matches.subcommand_matches(&id) {
        f(v.value_of("name").unwrap());
    }
}

pub fn handle_run_data_query(id: &str, matches: &clap::ArgMatches, f: &Fn(&str, &crate::types::RunId) -> (), g: &Fn() -> ()) {
    if let Some(v) = matches.subcommand_matches(&id) {
        match parse_bench_id(&v, "run_id") {
            (Some(name), Some(run_id)) => {
                if let Some(parsed_run_id) = crate::io::parse_run_id(&name, &run_id) {
                    f(&name, &parsed_run_id);
                }
            }
            (None, None) => g(),
            (_, _) => unreachable!()
        }
    }
}

pub fn handle_multi_run_data_query(
    id: &str,
    matches: &clap::ArgMatches,
    f: &Fn(&str, crate::types::RunId, crate::types::RunId) -> (),
    g: &Fn() -> ()
)
{
    if let Some(v) = matches.subcommand_matches(&id) {
        match (v.value_of("name"), v.value_of("run_id_1"), v.value_of("run_id_2")) {
            (Some(name), Some(run_id_1), Some(run_id_2)) => {
                match (crate::io::parse_run_id(&name, &run_id_1), crate::io::parse_run_id(&name, &run_id_2)) {
                    (Some(parsed_run_id_1), Some(parsed_run_id_2)) => f(&name, parsed_run_id_1, parsed_run_id_2),
                    (_, _) => {}
                }
            }
            (None, None, None) => g(),
            (_, _, _) => unreachable!()
        }
    }
}
