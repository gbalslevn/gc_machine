use circuit_macro::{circuit, circuit_fn};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gc_machine::circuit_builder::{CircuitBuild, CircuitBuilder};
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
// start venv and then run `parse_criterion.py` after `cargo bench`

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

#[circuit_fn]
fn produce_stacked_conditional(garbler_input: u64, evaluator_input: u64) -> u64 {
    if garbler_input == evaluator_input {
        garbler_input + evaluator_input
    } else {
        garbler_input + evaluator_input
    }
}
#[circuit_fn(naive_stack=true)]
fn produce_naive_conditional(garbler_input: u64, evaluator_input: u64) -> u64 {
    if garbler_input == evaluator_input {
        garbler_input + evaluator_input
    } else {
        garbler_input + evaluator_input
    }
}

fn bench_naive_conditional(c: &mut Criterion) {
    let cb = circuit!(produce_naive_conditional); // or in some other way provide a relevant circuit_build
    bench_optimisation_function(c, "naive_conditional", Garbler::new(HalfGatesGateGen::new()), HalfGatesEvaluator::new(), cb);
}
fn bench_stacked_conditional(c: &mut Criterion) {
    let cb = circuit!(produce_stacked_conditional); // or in some other way provide a relevant circuit_build
    bench_optimisation_function(c, "stacked_conditional", Garbler::new(HalfGatesGateGen::new()), HalfGatesEvaluator::new(), cb);
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
    
    let serialized_circuit = postcard::to_allocvec(&circuit).expect("serialization failed");
    let serialized_eval_input = postcard::to_allocvec(&eval_input).expect("serialization failed");
    let protocol_bytes = serialized_circuit.len() + serialized_eval_input.len();

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

    write_bench_metrics(optimisation_name, protocol_bytes, &garble_mem, &eval_mem, garble_insns, eval_insns);
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
        for _ in 0..gates_to_build / 2 + 1 {
            circuit_builder.build_xor(&input_a[0], &input_b[0]);
        }
    }
    if build_and {
        for _ in 0..gates_to_build / 2 + 1 {
            circuit_builder.build_and(&input_a[0], &input_b[0]);
        }
    }
    circuit_builder.get_circuit_build()
}


criterion_group!(xor_gates_bench, original_xor_gate, grr3_xor_gate, point_and_permute_xor_gate, free_xor_xor_gate, half_gates_xor_gate);
criterion_group!(and_gates_bench, original_and_gate, grr3_and_gate, point_and_permute_and_gate, free_xor_and_gate, half_gates_and_gate);
criterion_group!(function_bench, original_function, grr3_function, point_and_permute_function, free_xor_function, half_gates_function);
criterion_group!(function_and_bench, original_only_and_function, grr3_only_and_function, point_and_permute_only_and_function, free_xor_only_and_function, half_gates_only_and_function);
criterion_group!(function_xor_bench, original_only_xor_function, grr3_only_xor_function, point_and_permute_only_xor_function, free_xor_only_xor_function, half_gates_only_xor_function);
criterion_group!(conditional_bench, bench_naive_conditional, bench_stacked_conditional);
criterion_main!(function_bench, function_and_bench, function_xor_bench, conditional_bench);

