use std::collections::HashSet;
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gc_machine::circuit_builder::{BuildBlock, CircuitBuild, CircuitBuilder, WireBuild, get_input_wires};
use gc_machine::evaluator::evaluator::Evaluator;
use gc_machine::evaluator::free_xor_evaluator::FreeXOREvaluator;
use gc_machine::garbler::Garbler;
use gc_machine::evaluator::grr3_evaluator::GRR3Evaluator;
use gc_machine::evaluator::half_gates_evaluator::HalfGatesEvaluator;
use gc_machine::evaluator::original_evaluator::OriginalEvaluator;
use gc_machine::evaluator::point_and_permute_evaluator::PointAndPermuteEvaluator;
use gc_machine::gates::free_xor_gate_gen::FreeXORGateGen;
use gc_machine::gates::gate_gen::{GateType, GateGen};
use gc_machine::gates::grr3_gate_gen::GRR3GateGen;
use gc_machine::gates::half_gates_gate_gen::HalfGatesGateGen;
use gc_machine::gates::point_and_permute_gate_gen::PointAndPermuteGateGen;
use gc_machine::gates::original_gate_gen::OriginalGateGen;
use gc_machine::wires::wire_gen::WireGen;
use gc_machine::{global_mem_alloc};
use num_bigint::{ToBigUint};
use crate::bench_utils::{InsnCounter, get_memory, write_bench_metrics};

// run with `cargo bench`
// report available in /target/criterion/report

// python script can construct a comparison report
// start venv and then run `parse_benchmarks.py` after `cargo bench`

// To compensate for unreliable hardware, each bench is ran for x samples and then the average is taken.
// Criterion's defaults are:
// 100 samples
// Each sample runs the function enough times to fill a ~5ms measurement window
// 3 seconds warm-up where it measures how many times it needs to run the function to fill the window
// So for a fast function it may run thousands of iterations per sample; for a slow function it may run just once per sample.

#[path = "bench_utils.rs"] 
mod bench_utils;

// *********** BENCH FOR XOR GATE ***********
fn original_xor_gate(c: &mut Criterion) {
    bench_optimisation_gate(
        c,
        "original xor",
        GateType::XOR,
        OriginalGateGen::new(),
        OriginalEvaluator::new(),
    );
}

fn point_and_permute_xor_gate(c: &mut Criterion) {
    bench_optimisation_gate(
        c,
        "point and permute xor",
        GateType::XOR,
        PointAndPermuteGateGen::new(),
        PointAndPermuteEvaluator::new(),
    );
}

fn free_xor_xor_gate(c: &mut Criterion) {
    bench_optimisation_gate(
        c,
        "free xor xor",
        GateType::XOR,
        FreeXORGateGen::new(),
        FreeXOREvaluator::new()
    )
}

fn grr3_xor_gate(c: &mut Criterion) {
    bench_optimisation_gate(
        c,
        "grr3 xor",
        GateType::XOR,
        GRR3GateGen::new(),
        GRR3Evaluator::new(),
    );
}

fn half_gates_xor_gate(c: &mut Criterion) {
    bench_optimisation_gate(
        c,
        "half gates xor",
        GateType::XOR,
        HalfGatesGateGen::new(),
        HalfGatesEvaluator::new(),
    );
}

// *********** BENCH FOR AND GATE ***********
fn original_and_gate(c: &mut Criterion) {
    bench_optimisation_gate(
        c,
        "original and",
        GateType::AND,
        OriginalGateGen::new(),
        OriginalEvaluator::new(),
    );
}

fn point_and_permute_and_gate(c: &mut Criterion) {
    bench_optimisation_gate(
        c,
        "point and permute and",
        GateType::AND,
        PointAndPermuteGateGen::new(),
        PointAndPermuteEvaluator::new(),
    );
}

fn free_xor_and_gate(c: &mut Criterion) {
    bench_optimisation_gate(
        c,
        "free xor and",
        GateType::AND,
        FreeXORGateGen::new(),
        FreeXOREvaluator::new(),
    );
}


fn grr3_and_gate(c: &mut Criterion) {
    bench_optimisation_gate(
        c,
        "grr3 and",
        GateType::AND,
        GRR3GateGen::new(),
        GRR3Evaluator::new(),
    );
}

fn half_gates_and_gate(c: &mut Criterion) {
    bench_optimisation_gate(
        c,
        "half gates and",
        GateType::AND,
        HalfGatesGateGen::new(),
        HalfGatesEvaluator::new(),
    );
}

// *********** BENCH FOR A FUNCTION ***********
fn original_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "original",
        Garbler { gate_gen: OriginalGateGen::new() },
        OriginalEvaluator::new(),
        get_test_circuit(true, true)
    );
}

fn point_and_permute_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "point and permute",
        Garbler { gate_gen: PointAndPermuteGateGen::new() },
        PointAndPermuteEvaluator::new(),
        get_test_circuit(true, true)
    );
}

fn grr3_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "grr3",
        Garbler { gate_gen: GRR3GateGen::new() },
        GRR3Evaluator::new(),
        get_test_circuit(true, true)
    );
}

fn free_xor_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "free xor",
        Garbler { gate_gen: FreeXORGateGen::new() },
        FreeXOREvaluator::new(),
        get_test_circuit(true, true)
    );
}

fn half_gates_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "half gates",
        Garbler { gate_gen: HalfGatesGateGen::new() },
        HalfGatesEvaluator::new(),
        get_test_circuit(true, true)
    );
}

// *********** BENCH FOR A FUNCTION CONTAINING ONLY AND ***********
fn original_only_and_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "original - only AND",
        Garbler { gate_gen: OriginalGateGen::new() },
        OriginalEvaluator::new(),
        get_test_circuit(false, true)
    );
}

fn point_and_permute_only_and_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "point and permute - only AND",
        Garbler { gate_gen: PointAndPermuteGateGen::new() },
        PointAndPermuteEvaluator::new(),
        get_test_circuit(false, true)
    );
}

fn grr3_only_and_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "grr3 - only AND",
        Garbler { gate_gen: GRR3GateGen::new() },
        GRR3Evaluator::new(),
        get_test_circuit(false, true)
    );
}

fn free_xor_only_and_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "free xor - only AND",
        Garbler { gate_gen: FreeXORGateGen::new() },
        FreeXOREvaluator::new(),
        get_test_circuit(false, true)
    );
}

fn half_gates_only_and_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "half gates - only AND",
        Garbler { gate_gen: HalfGatesGateGen::new() },
        HalfGatesEvaluator::new(),
        get_test_circuit(false, true)
    );
}

// *********** BENCH FOR A FUNCTION CONTAINING ONLY XOR ***********
fn original_only_xor_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "original - only XOR",
        Garbler { gate_gen: OriginalGateGen::new() },
        OriginalEvaluator::new(),
        get_test_circuit(true, false)
    );
}

fn point_and_permute_only_xor_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "point and permute - only XOR",
        Garbler { gate_gen: PointAndPermuteGateGen::new() },
        PointAndPermuteEvaluator::new(),
        get_test_circuit(true, false)
    );
}

fn grr3_only_xor_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "grr3 - only XOR",
        Garbler { gate_gen: GRR3GateGen::new() },
        GRR3Evaluator::new(),
        get_test_circuit(true, false)
    );
}

fn free_xor_only_xor_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "free xor - only XOR",
        Garbler { gate_gen: FreeXORGateGen::new() },
        FreeXOREvaluator::new(),
        get_test_circuit(true, false)
    );
}

fn half_gates_only_xor_function(c: &mut Criterion) {
    bench_optimisation_function(
        c,
        "half gates - only XOR",
        Garbler { gate_gen: HalfGatesGateGen::new() },
        HalfGatesEvaluator::new(),
        get_test_circuit(true, false)
    );
}

// Finds the sweetspot between naive and stacked
fn bench_equal_naive_conditional(c: &mut Criterion) {
    let naive_circuit = get_conditional_test_circuit(false, 250, 1000);
    bench_optimisation_function(c, "naive_conditional - equal", Garbler::new(HalfGatesGateGen::new()), HalfGatesEvaluator::new(), naive_circuit);
}
fn bench_equal_stacked_conditional(c: &mut Criterion) {
    let stacked_circuit = get_conditional_test_circuit(true, 250, 1000);
    bench_optimisation_function(c, "stacked_conditional - equal", Garbler::new(HalfGatesGateGen::new()), HalfGatesEvaluator::new(), stacked_circuit);
}

// Show naive winning
fn bench_winning_naive_conditional(c: &mut Criterion) {
    let naive_circuit = get_conditional_test_circuit(false, 2000, 1000);
    bench_optimisation_function(c, "naive_conditional - winning", Garbler::new(HalfGatesGateGen::new()), HalfGatesEvaluator::new(), naive_circuit);
}
fn bench_loosing_stacked_conditional(c: &mut Criterion) {
    let stacked_circuit = get_conditional_test_circuit(true, 2000, 1000);
    bench_optimisation_function(c, "stacked_conditional - loosing", Garbler::new(HalfGatesGateGen::new()), HalfGatesEvaluator::new(), stacked_circuit);
}

// Show stacked winning
fn bench_loosing_naive_conditional(c: &mut Criterion) {
    let naive_circuit = get_conditional_test_circuit(false, 2, 1000);
    bench_optimisation_function(c, "naive_conditional - loosing", Garbler::new(HalfGatesGateGen::new()), HalfGatesEvaluator::new(), naive_circuit);
}
fn bench_winning_stacked_conditional(c: &mut Criterion) {
    let stacked_circuit = get_conditional_test_circuit(true, 2, 1000);
    bench_optimisation_function(c, "stacked_conditional - winning", Garbler::new(HalfGatesGateGen::new()), HalfGatesEvaluator::new(), stacked_circuit);
}

fn bench_optimisation_function<E, G>(
    c: &mut Criterion,
    optimisation_name: &str,
    mut garbler: Garbler<G>,
    mut evaluator: E,
    cb :CircuitBuild
)
where
    G: GateGen,
    E: Evaluator,
    GateType: Clone, {
    
    let garbler_input = garbler.create_circuit_input(&0.to_biguint().unwrap(), cb.required_input_bits);
    let (eval_input, eval_keys) = evaluator.create_circuit_input(&0.to_biguint().unwrap(), cb.required_input_bits);
    let instruction_counter = InsnCounter::new();

    // *** Bench garbling ***
    let (_, garble_mem) = get_memory(|| {
        garbler.create_circuit(&cb, &garbler_input, &eval_input);
    }, global_mem_alloc::GLOBAL);

    c.bench_function(&format!("{optimisation_name} create circuit"), |b| b.iter(|| {
        garbler.create_circuit(&cb, &garbler_input, &eval_input);
    }));

    let (_, garble_insns) = instruction_counter.measure(|| {
        garbler.create_circuit(&cb, &garbler_input, &eval_input);
    });

    // *** Get required bytes to complete protocol ***
    garbler.gate_gen.reset_index();
    let circuit = garbler.create_circuit(&cb, &garbler_input, &eval_input);
    
    let serialized_circuit_material = postcard::to_allocvec(&circuit.material).expect("serialization failed");
    println!("circuit material {}", serialized_circuit_material.len());
    let circuit_bytes: Vec<u8> = circuit.material.clone()
        .into_iter()
        .flatten() // Turns Vec<Vec<BigUint>> into an iterator of BigUint
        .flat_map(|big_uint| big_uint.to_bytes_be()) // Convert each to big-endian bytes
        .collect();
    // assert_eq!(serialized_circuit_material.len(), circuit_bytes.len());

    // *** Bench evaluating ***
    let (_, eval_mem) = get_memory(|| {
        evaluator.reset_index();
        evaluator.evaluate_circuit(&cb, &circuit, &eval_keys);
    }, global_mem_alloc::GLOBAL);

    c.bench_function(&format!("{optimisation_name} evaluate circuit"), |b| b.iter(|| {
        evaluator.reset_index();
        evaluator.evaluate_circuit(&cb, &circuit, &eval_keys);
    }));

    let (_, eval_insns) = instruction_counter.measure(|| {
        evaluator.reset_index();
        evaluator.evaluate_circuit(&cb, &circuit, &eval_keys);
    });

    write_bench_metrics(optimisation_name, circuit_bytes.len(), &garble_mem, &eval_mem, garble_insns, eval_insns);
}


fn bench_optimisation_gate<G, E>(
    c: &mut Criterion,
    optimisation_name: &str,
    gate_type: GateType,
    mut gate_gen: G,
    mut evaluator: E,
)
where
    G: GateGen,
    E: Evaluator,
    GateType: Clone,
{
    let wi = gate_gen.get_wire_gen().generate_input_wire();
    let wj = gate_gen.get_wire_gen().generate_input_wire();

    // *** Bench garbling ***
    get_memory(|| {
        gate_gen.generate_gate(gate_type.clone(), wi.clone(), wj.clone());
    }, global_mem_alloc::GLOBAL);

    c.bench_function(&format!("{optimisation_name} gate garbling"), |b| b.iter(|| {
        gate_gen.generate_gate(
            black_box(gate_type),
            black_box(wi.clone()),
            black_box(wj.clone()),
        );
    }));

    gate_gen.reset_index();
    let gate = gate_gen.generate_gate(gate_type.clone(), wi, wj);
    // *** Bench evaluating ***
    get_memory(|| {
        evaluator.reset_index();
        evaluator.evaluate_gate(&gate.wi.w0(), &gate.wj.w1(), &gate.gate_type, &gate.table);
    }, global_mem_alloc::GLOBAL);
    
    c.bench_function(&format!("{optimisation_name} gate evaluation"), |b| b.iter(|| {
        evaluator.reset_index();
        evaluator.evaluate_gate(
            black_box(&gate.wi.w0()),
            black_box(&gate.wj.w1()),
            black_box(&gate.gate_type),
            black_box(&gate.table),
        );
    }));
}

fn get_test_circuit(build_xor : bool, build_and: bool) -> CircuitBuild {
    let mut circuit_builder = CircuitBuilder::new();
    let gates_to_build = 10000;
    let required_input_bits = 1;     // Garbler and Evaluator only provides an input of 1 bit, as the only thing that matters in this benchmark, is the amount of gates being created and evaluated. The underlying input values are not of interest. 
    let (input_a, input_b) = circuit_builder.set_input_wires(required_input_bits);
    if build_xor {
        for _ in 0..gates_to_build / 2 {
            circuit_builder.build_xor(&input_a[0], &input_b[0]);
        }
    }
    if build_and {
        for _ in 0..gates_to_build / 2 {
            circuit_builder.build_and(&input_a[0], &input_b[0]);
        }
    }
    circuit_builder.get_circuit_build()
}

fn get_conditional_test_circuit(stacked : bool, input_length : usize, gates_in_each_subcircuit : usize) -> CircuitBuild {
    let total_gates = gates_in_each_subcircuit * 2;
    if input_length > total_gates * 2 { 
        panic!("The input length is not possible if each gate in the circuit has exactly two inputs. Only possible if input is a variable, which this function cannot create.")
    }
    let mut circuit_builder = CircuitBuilder::new();
    let mut true_gates = vec![];
    let mut false_gates = vec![];
    let (input_a, input_b) = circuit_builder.set_input_wires(input_length as u64);
    let dummy_wire = input_a[0].clone();

    // Puts gates_in_each_subcircuit number AND gates for both subcircuits and that the amount of input wires is the same as input_length
    if stacked {
        for i in 0..gates_in_each_subcircuit {
            let and_gate;
            // ensure gates in each block uses the right amount of unique input wires, then if required, add redudent gates if input_length < gates_in_each_subcircuit
            if i < input_length / 2 {
                and_gate = circuit_builder.build_and(&input_a[i], &input_b[i]); 
            } else {
                and_gate = circuit_builder.build_and(&input_a[0], &input_b[0]);
            }
            true_gates.push(and_gate.builds[0].clone());
            false_gates.push(and_gate.builds[0].clone());
        }
        let mut true_block = BuildBlock { builds : true_gates.clone(), output : vec![dummy_wire.clone()]};
        let mut false_block = BuildBlock { builds : false_gates.clone(), output : vec![dummy_wire.clone()]};
        circuit_builder.build_stacked_if(&input_a[0], &mut false_block, &mut true_block);

        assert_eq!(true_gates.len(), gates_in_each_subcircuit);
        assert_eq!(false_gates.len(), gates_in_each_subcircuit);
        let true_block_inputs = get_input_wires(true_block);
        let false_block_inputs = get_input_wires(false_block);
        let combined_input: HashSet<WireBuild> = true_block_inputs.clone().into_iter().chain(false_block_inputs.clone().into_iter()).collect();
        assert_eq!(combined_input.len(), input_length);
    } else {
        // Puts gates_in_each_subcircuit for both true and false branch.  
        for _ in 0..gates_in_each_subcircuit {
            circuit_builder.build_and(&dummy_wire, &dummy_wire); // build for true
            circuit_builder.build_and(&dummy_wire, &dummy_wire); // build for false
        }
        let true_block = BuildBlock { builds : vec![], output : vec![dummy_wire.clone()]}; // No need to provide the builds as they will be evaluated anyway in the naive as they are a part of the circuit. 
        let false_block = BuildBlock { builds : vec![], output : vec![dummy_wire.clone()]};
        circuit_builder.build_if(&input_a[0], &false_block, &true_block);
    }
    circuit_builder.get_circuit_build()
}

criterion_group!(
    name = xor_gates_bench;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = original_xor_gate, grr3_xor_gate, point_and_permute_xor_gate, free_xor_xor_gate, half_gates_xor_gate
);
criterion_group!(
    name = and_gates_bench;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = original_and_gate, grr3_and_gate, point_and_permute_and_gate, free_xor_and_gate, half_gates_and_gate
);
criterion_group!(
    name = function_bench;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = original_function, grr3_function, point_and_permute_function, free_xor_function, half_gates_function
);
criterion_group!(
    name = function_and_bench;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = original_only_and_function, grr3_only_and_function, point_and_permute_only_and_function, free_xor_only_and_function, half_gates_only_and_function
);
criterion_group!(
    name = function_xor_bench;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = original_only_xor_function, grr3_only_xor_function, point_and_permute_only_xor_function, free_xor_only_xor_function, half_gates_only_xor_function
);
criterion_group!(
    name = conditional_bench;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = bench_equal_naive_conditional, bench_equal_stacked_conditional, bench_loosing_naive_conditional, bench_winning_stacked_conditional, bench_winning_naive_conditional, bench_loosing_stacked_conditional
);
criterion_group!(
    name = conditional_bench_test;
    config = Criterion::default().measurement_time(Duration::from_secs(60));
    targets = bench_loosing_naive_conditional, bench_winning_stacked_conditional
);
// criterion_main!(function_bench, function_and_bench, function_xor_bench, conditional_bench);
criterion_main!(conditional_bench);

