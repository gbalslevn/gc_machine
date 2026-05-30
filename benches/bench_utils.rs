use stats_alloc::{Region, StatsAlloc};
use std::alloc::{System};

// maybe use this to run benching, cargo bench -- --test-threads=1

// Measures amount of heap memory used for the provided action
pub fn get_memory<F, R>(function: F, global: &StatsAlloc<System>) -> (R, Stats)
where
    F: FnOnce() -> R,
{
    let reg = Region::new(&global);
    let result = function();
    let stats = reg.change();
    (result, stats)
}

use std::collections::HashMap;
use stats_alloc::Stats;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct BenchMetrics {
    protocol_bytes: usize,
    garble_bytes_allocated: usize,
    garble_net_bytes: isize,
    eval_bytes_allocated: usize,
    eval_net_bytes: isize,
}

// Write metrics to a file
pub fn write_bench_metrics(name: &str, protocol_bytes: usize, garble: &Stats, eval: &Stats) {
    let path = std::path::Path::new("target/criterion/bench_metrics.json");
    let mut map: HashMap<String, BenchMetrics> = path.exists()
        .then(|| serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap_or_default())
        .unwrap_or_default();

    map.insert(name.to_string(), BenchMetrics {
        protocol_bytes,
        garble_bytes_allocated: garble.bytes_allocated,
        garble_net_bytes: garble.bytes_allocated as isize - garble.bytes_deallocated as isize,
        eval_bytes_allocated: eval.bytes_allocated,
        eval_net_bytes: eval.bytes_allocated as isize - eval.bytes_deallocated as isize,
    });

    std::fs::create_dir_all("target/criterion").unwrap();
    std::fs::write(path, serde_json::to_string_pretty(&map).unwrap()).unwrap();
}