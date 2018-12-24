// Module containing core types representing
// - Info coming from google/benchmark (runs, machine info)
// - Source diffs between changes
// - Comments between changes
//
// Much of this information is serialized into JSON files in the
// config directory for this program, so most derive it from serde.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO: Cleanup naming of types, seems a bit inconsistent

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BenchResult {
    pub name: String,
    pub iterations: i64,
    pub real_time: f64,
    pub cpu_time: f64,
    pub time_unit: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CpuCacheInfo {
    pub level: i64,
    pub size: i64,
    pub num_sharing: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnvironmentInfo {
    pub date: String,
    pub executable: String,
    pub num_cpus: i64,
    pub mhz_per_cpu: i64,
    pub cpu_scaling_enabled: bool,
    pub caches: Vec<CpuCacheInfo>,
    pub library_build_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BenchRunResult {
    pub context: Option<EnvironmentInfo>,
    pub benchmarks: Vec<BenchResult>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IndividualBenchInfo {
    // TODO: Update context per machine, just in case :)
    // TODO: Add git changes
    pub context: Option<EnvironmentInfo>,
    pub commentary: Vec<String>,
    pub benchmarks: Vec<Vec<BenchResult>>,
    pub source_hashes: Vec<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BenchHeader {
    // TODO: Make this a Path, not a string
    pub source_root: String,
    pub source_bin: String,
    pub description: String,
}

pub type BenchId = String;
pub type TopLevelBenchInfo = HashMap<BenchId, BenchHeader>;
