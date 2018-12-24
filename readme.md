![build status](https://travis-ci.org/superfunc/benchviz.svg?branch=master)
> WIP 

A tool for cataloging, annotating & plotting C++ benchmarks
written with [google/benchmark](github.com/google/benchmark).

Rundown:
- [x] Running benchmarks.
- [x] Saving results to json.
- [x] Reading config/results from json.
- [x] CLI parsing setup.
- [x] Caching source changes.
- [x] Generating git diffs.
- [ ] Ensure the binary is built with gbench?.
- [ ] Add more error handling around erroneous output.
- [ ] Clean up redundancy in io.rs around reading configs.
- [ ] Plotting.
    - [x] Add basic SVG generation for plotting.
    - [-] Add CLI Plotting default.
    - [ ] Add legend explaining colors for SVG output.
    - [ ] Generating full markdown or html reports.
