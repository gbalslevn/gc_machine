use stats_alloc::{Region, StatsAlloc};
use std::alloc::{System};

// maybe use this to run benching, cargo bench -- --test-threads=1

// Measures amount of heap memory used for the provided action
pub fn get_memory<F, R>(function: F, global : &StatsAlloc<System>) -> R 
where 
    F: FnOnce() -> R,
{
    let reg = Region::new(&global);
    
    let result = function();
    
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