use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gc_machine::evaluator::evaluator::Evaluator;
use gc_machine::evaluator::grr3_evaluator::GRR3Evaluator;
use gc_machine::evaluator::original_evaluator::OriginalEvaluator;
use gc_machine::evaluator::point_and_permute_evaluator::PointAndPermuteEvaluator;
use gc_machine::gates::gates::{GateType, Gates};
use gc_machine::gates::grr3_gates::GRR3Gates;
use gc_machine::gates::point_and_permute_gates::PointAndPermuteGates;
use gc_machine::gates::original_gates::OriginalGates;
use gc_machine::wires::grr3_wires::GRR3Wires;
use gc_machine::wires::original_wires::OriginalWires;
use gc_machine::wires::point_and_permute_wires::PointAndPermuteWires;
use gc_machine::wires::wires::Wires;
use gc_machine::global_mem_alloc;
use num_bigint::BigUint;

#[path = "bench_utils.rs"] 
mod bench_utils;

pub fn original_xor_gate(c: &mut Criterion) {
    let gate_type = GateType::XOR;

    let wire_gen = OriginalWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = OriginalGates::new(wire_gen);
    let mut gt = gate_gen.generate_gate(gate_type, wi, wj, BigUint::ZERO);

    // *** Bench garbling ***
    bench_utils::get_memory(|| {
        gt = gate_gen.generate_gate(gate_type.clone(), gt.wi.clone(), gt.wj.clone(), gt.gate_id.clone());
    }, global_mem_alloc::GLOBAL);
    
    c.bench_function("original xor gate garbling", |b| b.iter(|| {
        gt = gate_gen.generate_gate(black_box(gt.gate_type), black_box(gt.wi.clone()), black_box(gt.wj.clone()), black_box(gt.gate_id.clone()));
    })); // black_box prevents compiler from optimizing the function away but taking the value value, acting like it uses it, preventing the optimizer from seeing through the function. Especially usefull when calling function 1000 times

    // *** Bench evaluating ***
    bench_utils::get_memory(|| {
    }, global_mem_alloc::GLOBAL);
    
    c.bench_function("original xor gate evaluation", |b| b.iter(|| {
        OriginalEvaluator::evaluate_gate(black_box(&gt.wi.w0()), black_box(&gt.wj.w1()), black_box( &gt.gate_type), black_box(&gt.gate_id), black_box(&gt.table));
    }));
}

pub fn grr3_xor_gate(c: &mut Criterion) {
    let gate_type = GateType::XOR;
    let wire_gen = GRR3Wires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = GRR3Gates::new(wire_gen);
    let mut gt = gate_gen.generate_gate(gate_type.clone(), wi.clone(), wj.clone(), BigUint::ZERO);

    // *** Bench garbling ***
    bench_utils::get_memory(|| {
        gt = gate_gen.generate_gate(gt.gate_type, gt.wi.clone(), gt.wj.clone(), gt.gate_id.clone());
    }, global_mem_alloc::GLOBAL);
    
    c.bench_function("grr3 xor gate garbling", |b| b.iter(|| {
        gt = gate_gen.generate_gate(black_box(gt.gate_type), black_box(gt.wi.clone()), black_box(gt.wj.clone()), black_box(gt.gate_id.clone()));
    })); 

    // *** Bench evaluating ***
    bench_utils::get_memory(|| {
        GRR3Evaluator::evaluate_gate(black_box(&gt.wi.w0()), black_box(&gt.wj.w1()), black_box( &gt.gate_type), black_box(&gt.gate_id), black_box(&gt.table));
    }, global_mem_alloc::GLOBAL);
    
    c.bench_function("grr3 xor gate evaluation", |b| b.iter(|| {
        GRR3Evaluator::evaluate_gate(black_box(&gt.wi.w0()), black_box(&gt.wj.w1()), black_box( &gt.gate_type), black_box(&gt.gate_id), black_box(&gt.table));
    }));
}

pub fn point_and_permute_xor_gate(c: &mut Criterion) {
    let gate_type = GateType::XOR;
    let wire_gen = PointAndPermuteWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = PointAndPermuteGates::new(wire_gen);
    let mut gt = gate_gen.generate_gate(gate_type.clone(), wi.clone(), wj.clone(), BigUint::ZERO);

    // *** Bench garbling ***
    bench_utils::get_memory(|| {
        gt = gate_gen.generate_gate(gate_type.clone(), wi.clone(), wj.clone(), BigUint::ZERO);
    }, global_mem_alloc::GLOBAL);
    
    c.bench_function("grr3 xor gate garbling", |b| b.iter(|| {
        gt = gate_gen.generate_gate(black_box(gt.gate_type), black_box(gt.wi.clone()), black_box(gt.wj.clone()), black_box(gt.gate_id.clone()));
    })); 

    // *** Bench evaluating ***
    bench_utils::get_memory(|| {
        PointAndPermuteEvaluator::evaluate_gate(black_box(&gt.wi.w0()), black_box(&gt.wj.w1()), black_box( &gt.gate_type), black_box(&gt.gate_id), black_box(&gt.table));
    }, global_mem_alloc::GLOBAL);
    
    c.bench_function("grr3 xor gate evaluation", |b| b.iter(|| {
        PointAndPermuteEvaluator::evaluate_gate(black_box(&gt.wi.w0()), black_box(&gt.wj.w1()), black_box( &gt.gate_type), black_box(&gt.gate_id), black_box(&gt.table));
    }));
}

criterion_group!(benches, original_xor_gate, grr3_xor_gate);
criterion_main!(benches);

