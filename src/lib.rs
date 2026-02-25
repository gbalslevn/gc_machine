// The root of our library to keep track of all files. 
// File to declare all modules

pub mod crypto_utils;
pub mod gates;
pub mod wires;
pub mod evaluator;
pub mod global_mem_alloc;
pub mod ot;
pub mod circuit_builder;
pub mod garbler;

#[cfg(test)] // Only compile when testing
mod unit_tests; // Finds unit tests folder