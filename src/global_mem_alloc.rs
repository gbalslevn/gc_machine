use stats_alloc::{StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

// Get the global allocator of the program and run it with custom code. In this case INSTRUMENTED_SYSTEM for benchmarking. 

#[global_allocator]
pub static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;