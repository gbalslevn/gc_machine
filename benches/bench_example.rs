use criterion::{black_box, criterion_group, criterion_main, Criterion};
use stats_alloc::{INSTRUMENTED_SYSTEM, StatsAlloc};
use std::alloc::{System};

#[path = "bench_utils.rs"] 
mod bench_utils;

// Benchmarking
// Reports can be found under target/criterion

#[global_allocator] // Dont use default system memory allocator but a custom piece of code. In this case the INSTRUMENTED_SYSTEM
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => n,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn heap_usage(capacity : usize) {
    let x: Vec<u8> = Vec::with_capacity(capacity);
}

pub fn criterion_example_benchmark(c: &mut Criterion) {
    let result = bench_utils::get_memory(||{ // This will be benched as 0 bytes used, as no heap is used 
        fibonacci(20)
    }, GLOBAL);
    let result = bench_utils::get_memory(|| {
        heap_usage(10)
    }, GLOBAL);
    c.bench_function("fib 20", |b| b.iter(|| {
        fibonacci(black_box(20));
    })); // black_box prevents compiler from optimizing the function away but taking the value value, acting like it uses it, preventing the optimizer from seeing through the function.
}

// To measure compute (cycles and instruction) use criterion-perf-events, only available on Linux. 

criterion_group!(benches, criterion_example_benchmark);
criterion_main!(benches);