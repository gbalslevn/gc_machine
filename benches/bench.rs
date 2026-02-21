use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gc_machine::evaluator::evaluator::Evaluator;
use gc_machine::evaluator::grr3_evaluator::GRR3Evaluator;
use gc_machine::evaluator::original_evaluator::OriginalEvaluator;
use gc_machine::evaluator::point_and_permute_evaluator::PointAndPermuteEvaluator;
use gc_machine::gates::gates::{GateType, Gates};
use gc_machine::gates::grr3_gates::GRR3Gates;
use gc_machine::gates::point_and_permute_gates::PointAndPermuteGates;
use gc_machine::gates::original_gates::OriginalGates;
use gc_machine::wires::wires::Wires;
use gc_machine::global_mem_alloc;
use num_bigint::BigUint;

#[path = "bench_utils.rs"] 
mod bench_utils;

pub fn original_xor_gate(c: &mut Criterion) {
    let gate_type = &GateType::XOR;
    let mut gt = OriginalGates::new(gate_type, BigUint::ZERO);

    // *** Bench garbling ***
    bench_utils::get_memory(|| {
        gt = OriginalGates::new(&gate_type, BigUint::ZERO);
    }, global_mem_alloc::GLOBAL);
    
    c.bench_function("original xor gate garbling", |b| b.iter(|| {
        gt = OriginalGates::new(black_box(&gate_type), black_box(BigUint::ZERO));
    })); // black_box prevents compiler from optimizing the function away but taking the value value, acting like it uses it, preventing the optimizer from seeing through the function. Especially usefull when calling function 1000 times

    // *** Bench evaluating ***
    bench_utils::get_memory(|| {
    }, global_mem_alloc::GLOBAL);
    
    c.bench_function("original xor gate evaluation", |b| b.iter(|| {
        OriginalEvaluator::evaluate_gate(black_box(&gt.wi.w0()), black_box(&gt.wj.w1()), black_box( &BigUint::ZERO), black_box(gate_type), black_box(&gt));
    }));
}

pub fn grr3_xor_gate(c: &mut Criterion) {
    let gate_type = &GateType::XOR;
    let mut gt = GRR3Gates::new(gate_type, BigUint::ZERO);

    // *** Bench garbling ***
    bench_utils::get_memory(|| {
        gt = GRR3Gates::new(&gate_type, BigUint::ZERO);
    }, global_mem_alloc::GLOBAL);
    
    c.bench_function("grr3 xor gate garbling", |b| b.iter(|| {
        gt = GRR3Gates::new(black_box(&gate_type), black_box(BigUint::ZERO));
    })); 

    // *** Bench evaluating ***
    bench_utils::get_memory(|| {
        GRR3Evaluator::evaluate_gate(black_box(&gt.wi.w0()), black_box(&gt.wj.w1()), black_box( &BigUint::ZERO), black_box(gate_type), black_box(&gt));
    }, global_mem_alloc::GLOBAL);
    
    c.bench_function("grr3 xor gate evaluation", |b| b.iter(|| {
        GRR3Evaluator::evaluate_gate(black_box(&gt.wi.w0()), black_box(&gt.wj.w1()), black_box( &BigUint::ZERO), black_box(gate_type), black_box(&gt));
    }));
}

pub fn point_and_permute_xor_gate(c: &mut Criterion) {
    let gate_type = &GateType::XOR;
    let mut gt = PointAndPermuteGates::new(gate_type, BigUint::ZERO);

    // *** Bench garbling ***
    bench_utils::get_memory(|| {
        gt = PointAndPermuteGates::new(black_box(&gate_type), black_box(BigUint::ZERO));
    }, global_mem_alloc::GLOBAL);
    
    c.bench_function("point and permute xor gate garbling", |b| b.iter(|| {
        gt = PointAndPermuteGates::new(black_box(&gate_type), black_box(BigUint::ZERO));
    })); 

    // *** Bench evaluating ***
    bench_utils::get_memory(|| {
        PointAndPermuteEvaluator::evaluate_gate(black_box(&gt.wi.w0()), black_box(&gt.wj.w1()), black_box( &BigUint::ZERO), black_box(gate_type), black_box(&gt));
    }, global_mem_alloc::GLOBAL);
    
    c.bench_function("point and permute xor gate evaluation", |b| b.iter(|| {
        PointAndPermuteEvaluator::evaluate_gate(black_box(&gt.wi.w0()), black_box(&gt.wj.w1()), black_box( &BigUint::ZERO), black_box(gate_type), black_box(&gt));
    }));
}

criterion_group!(benches, original_xor_gate, grr3_xor_gate);
criterion_main!(benches);

