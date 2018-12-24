mod config;
mod io;
mod types;

use clap::clap_app;

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
       (@subcommand run =>
          (about: "Run another iteration of a benchmark.")
          (@arg name: +required "Name of benchmark")))
    .get_matches();

    crate::config::ensure_initialized();

    if let Some(_) = matches.subcommand_matches("list") {
        crate::io::print_current_benchmarks();
    } else if let Some(_) = matches.subcommand_matches("new") {
        crate::io::create_new_individual_benchmark();
    } else if let Some(v) = matches.subcommand_matches("info") {
        crate::io::print_individual_bench_info(v.value_of("name").unwrap());
    } else if let Some(v) = matches.subcommand_matches("run") {
        crate::io::run_individual_benchmark(v.value_of("name").unwrap());
    } else if let Some(v) = matches.subcommand_matches("plot") {
        crate::io::plot_individual_benchmark(v.value_of("name").unwrap());
    }
}
