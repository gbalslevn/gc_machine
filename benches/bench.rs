use criterion::{black_box, criterion_group, criterion_main, Criterion};
use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator] // Dont use default system memory allocator but a custom piece of code. In this case the INSTRUMENTED_SYSTEM
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

// Benchmarking
// Reports can be found under target/criterion

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
    let result = get_memory(|| {
        fibonacci(20)
    });
    let result = get_memory(|| {
        heap_usage(10)
    });
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20)))); // black_box prevents compiler from optimizing the function away but taking the value value, acting like it uses it, preventing the optimizer from seeing through the function.
}

// Measures amount of heap memory used for the provided action
fn get_memory<F, R>(action: F) -> R 
where 
    F: FnOnce() -> R 
{
    let reg = Region::new(&GLOBAL);
    
    let result = action();
    
    let stats = reg.change();
    
    println!("--- Memory Report ---");
    println!("Allocations:   {}", stats.allocations);
    println!("Deallocations: {}", stats.deallocations);
    println!("Bytes Allocated:   {} B", stats.bytes_allocated);
    println!("Bytes Deallocated: {} B", stats.bytes_deallocated);
    println!("Net memory change: {}", stats.bytes_allocated as isize - stats.bytes_deallocated as isize);

    println!("---------------------");
    
    result
}

// To measure compute (cycles and instruction) use criterion-perf-events, only available on Linux. 

criterion_group!(benches, criterion_example_benchmark);
criterion_main!(benches);