// The root of our library to keep track of all files. 
// File to declare all modules

pub mod crypto_utils;

#[cfg(test)] // Only compile when testing
mod unit_tests; // Finds unit tests folder