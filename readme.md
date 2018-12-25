> WIP 

![https://travis-ci.org/superfunc/benchviz.svg?branch=master](https://travis-ci.org/superfunc/benchviz.svg?branch=master)

##### Description

A tool for cataloging, annotating & plotting C++ benchmarks
written with [google/benchmark](github.com/google/benchmark).

##### Platforms

Currently, its only being tested on macos, though it should work on
windows or linux. The only bit that may need some tweaking is the way
I'm launching git in a subprocess in src/io.rs.

##### Dependencies

This tool utilizes git to do diffing, if you don't have it installed
that feature will be turned off. Every other dependency is installed through
cargo.

#### TODO
- [x] Running benchmarks.
- [x] Saving results to json.
- [x] Reading config/results from json.
- [x] CLI parsing setup.
- [x] Caching source changes.
- [x] Generating git diffs.
- [ ] Ensure the binary is built with gbench?.
- [ ] Add more error handling around erroneous output.
- [x] Clean up redundancy in io.rs around reading configs.
- [ ] Handle git not being installed on the system.
- [ ] Plotting.
    - [x] Add basic SVG generation for plotting.
    - [ ] Add legend explaining colors for SVG output.
    - [ ] Generating full markdown or html reports.
