use stats_alloc::{Region, Stats, StatsAlloc};
use std::alloc::System;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Measures heap allocations for a closure.
pub fn get_memory<F, R>(f: F, global: &StatsAlloc<System>) -> (R, Stats)
where
    F: FnOnce() -> R,
{
    let reg = Region::new(global);
    let result = f();
    (result, reg.change())
}

/// Instruction counter — only functional on Linux.
pub struct InsnCounter {
    #[cfg(target_os = "linux")]
    inner: Option<perf::Counter>,
}

impl InsnCounter {
    pub fn new() -> Self {
        #[cfg(not(target_os = "linux"))]
        eprintln!("Warning: instruction counting is only supported on Linux.");

        Self {
            #[cfg(target_os = "linux")]
            inner: perf::Counter::new()
                .inspect_err(|e| eprintln!("perf_event_open failed: {e}")).ok(),
        }
    }

    /// Returns the result and instruction count (None on non-Linux or if perf unavailable).
    pub fn measure<F, R>(&self, f: F) -> (R, Option<u64>)
    where
        F: FnOnce() -> R,
    {
        #[cfg(target_os = "linux")]
        if let Some(ref c) = self.inner {
            let (r, n) = c.measure(f).expect("perf read failed");
            return (r, Some(n));
        }
        (f(), None)
    }
}

#[derive(Serialize, Deserialize, Default)]
struct BenchMetrics {
    protocol_bytes: usize,
    garble_bytes_allocated: usize,
    eval_bytes_allocated: usize,
    garble_instructions: Option<u64>,
    eval_instructions: Option<u64>,
}

pub fn write_bench_metrics(
    name: &str,
    protocol_bytes: usize,
    garble: &Stats,
    eval: &Stats,
    garble_instructions: Option<u64>,
    eval_instructions: Option<u64>,
) {
    const PATH: &str = "target/criterion/bench_metrics.json";

    let mut map: HashMap<String, BenchMetrics> = std::fs::read_to_string(PATH)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    map.insert(name.to_string(), BenchMetrics {
        protocol_bytes,
        garble_bytes_allocated: garble.bytes_allocated,
        eval_bytes_allocated: eval.bytes_allocated,
        garble_instructions,
        eval_instructions,
    });

    std::fs::write(PATH, serde_json::to_string_pretty(&map).unwrap()).unwrap();
}