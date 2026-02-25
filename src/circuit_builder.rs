use std::ops::Add;

use num_bigint::{BigUint, ToBigUint};
use crate::gates::gates::GateType;
use uuid::Uuid;

// Responsible for creating "recepies" for the gates which needs to be created by the garbler"

#[derive(Clone)]
pub struct WireBuild {
    id: Uuid,
    output_layer : BigUint // The layer which the wire was outputtet on
}

impl WireBuild {
    pub fn new(id: Uuid, output_layer : BigUint) -> Self {
        WireBuild { id, output_layer }
    }
    pub fn id(&self) -> &Uuid {
        &self.id
    }
    pub fn output_layer(&self) -> &BigUint {
        &self.output_layer
    }
}
pub struct GateBuild {
    gate_type : GateType,
    wi: WireBuild, 
    wj: WireBuild,
    wo: WireBuild,
}

impl GateBuild {
    pub fn new(gate_type: GateType, wi: WireBuild, wj : WireBuild, wo : WireBuild) -> Self {
        GateBuild { gate_type, wi, wj, wo }
    }
    pub fn gate_type(&self) -> &GateType {
        &self.gate_type
    }
    pub fn wi(&self) -> &WireBuild {
        &self.wi
    }
    pub fn wj(&self) -> &WireBuild {
        &self.wj
    }
    pub fn wo(&self) -> &WireBuild {
        &self.wo
    }
}

pub fn create_OR(input_wi : &WireBuild, input_wj : &WireBuild) -> Vec<GateBuild> {
    let xor_0 = create_gate(input_wi, input_wj, GateType::XOR);
    let and_0 = create_gate(input_wi, input_wj, GateType::AND);
    let xor_1 = create_gate(&xor_0.wo(), &and_0.wo(), GateType::XOR);
    vec![xor_0, and_0, xor_1]
}

// Creates gate with a new id and the output wire containing when the gate should be calculated
pub fn create_gate(wi : &WireBuild, wj : &WireBuild, gate_type : GateType) -> GateBuild {
    let id = Uuid::new_v4();
    // When it can be created when we have received both values
    let output_layer = wi.output_layer.clone().max(wj.output_layer.clone()); 
    let one = 1.to_biguint().unwrap();
    let wo = WireBuild::new(id, output_layer.add(one));
    GateBuild::new(gate_type, wi.clone(), wj.clone(), wo)
}

