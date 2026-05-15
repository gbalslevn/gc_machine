use crate::{circuit_builder::{BuildCount, BuildType, CircuitBuild, CircuitBuilder}};

#[test]
fn builds_are_sorted_by_increasing_output_layer() {
    let mut circuit_builder = CircuitBuilder::new();
    let (input_a, input_b) = circuit_builder.set_input_wires(10);
    circuit_builder.build_is_equal(&input_a, &input_b);
    let cb = circuit_builder.get_circuit_build();
    let builds = cb.get_builds();
    let mut current_output_layer = &1;
    
    for build in builds {
        assert!(build.ready_at_layer() >= &current_output_layer);
        current_output_layer = build.ready_at_layer()
    }
}

#[should_panic="Input wires not set"]
#[test]
fn panics_if_input_wires_not_set() {
    let mut circuit_builder = CircuitBuilder::new();
    circuit_builder.get_circuit_build();
}

#[test]
fn one_stacked_if_creates_2_branches() {
    let cb = get_stacked_if_build(); 
    let builds = cb.get_builds();
    assert_eq!(builds.len(), 1);
    assert_eq!(builds[0].get_type(), BuildType::Stack);
}

fn get_stacked_if_build() -> CircuitBuild {
    let mut builder = CircuitBuilder::new();
    builder.set_input_wires(1); // Need to set to avoid failing
    
    let inputs = builder.build_input_wires(2); 
    let cond = &inputs[0];
    let wi = &inputs[1];

    let mut and_0_block = builder.build_and(wi, wi);
    let mut and_1_block = builder.build_and(wi, wi);
    builder.build_stacked_if(cond, &mut and_0_block, &mut and_1_block);

    builder.get_circuit_build()
}

#[test]
fn can_count_material_in_builds() {
    let mut builder = CircuitBuilder::new();
    builder.set_input_wires(1);
    let input = builder.build_input_wires(1);
    
    let and_build = builder.build_and(&input[0], &input[0]);
    assert_eq!(and_build.builds.get_len(), 1);
}

// perhaps make test for stacked that c0 and c1 input, output and elements has equal length