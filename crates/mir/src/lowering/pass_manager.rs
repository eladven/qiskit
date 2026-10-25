use std::collections::BTreeSet;

use qiskit_circuit::dag_circuit::DAGCircuit;
use qiskit_circuit::operations::OperationRef;
use qiskit_passmanager::{PassContext, StaticPass};

pub struct MIRBlock {}

qiskit_util::dyn_types::static_dyn_typed!(MIRBlock);
impl qiskit_passmanager::IR for MIRBlock {}

pub struct MIRBlockCreator {}


impl MIRBlockCreator {
    pub fn new() -> Self {Self {}}

    pub fn create_qubit(&mut self, name: &str) {
        println!("create_qubit {}", name);
    }

    pub fn pauli_rotate(&mut self, qargs: &[String], paulis: &[String], angle: f64){
        println!("pauli_rotate: {:?} {:?} {:?}", qargs, paulis, angle);
    }

    pub fn pauli_measure(&mut self, qargs: &[String], paulis: &[String]){
        println!("pauli_measure: {:?} {:?}", qargs, paulis);
    }

    pub fn get_block(self)-> MIRBlock{
        MIRBlock{}
    }
}

fn pauli_label(x:bool, z:bool) -> &'static str {
    match (x, z) {
        (false, false) => "I",
        (true, false) => "X",
        (false, true) => "Z",
        (true, true) => "Y",
    }
}

pub fn lower(dag: &DAGCircuit) -> MIRBlock {
    let mut mir_creator = MIRBlockCreator::new();
    let mut qubits: BTreeSet<String> = BTreeSet::new();

    for node in dag.topological_op_nodes(false) {
        let instr = dag.dag()[node].unwrap_operation();
        let qargs = dag.get_qargs(instr.qubits);
        let qargs_ids : Vec<String> = qargs.iter().map(|q| q.0.to_string()).collect();
        for qid in &qargs_ids {
            qubits.insert(qid.clone());
        }
    }

    for qubit in &qubits{
        mir_creator.create_qubit(qubit);
    }

    for node in dag.topological_op_nodes(false) {
        let instr = dag.dag()[node].unwrap_operation();
        let qargs = dag.get_qargs(instr.qubits);
        let qargs_ids : Vec<String> = qargs.iter().map(|q| q.0.to_string()).collect();
        match instr.op.view() {
            OperationRef::PauliProductRotation(rotation) => {
                let paulis: Vec<String> = rotation.x.iter().zip(rotation.z.iter()).map(|(&x, &z)| pauli_label(x, z).to_string()).collect();
                let angle = match rotation.angle.try_float() {
                    Some(val) => val,
                    None => 0.0
                };
                mir_creator.pauli_rotate(&qargs_ids, &paulis, angle);
            }
            OperationRef::PauliProductMeasurement(measurment) => {
                let paulis: Vec<String> = measurment.x.iter().zip(measurment.z.iter()).map(|(&x, &z)| pauli_label(x, z).to_string()).collect();
                mir_creator.pauli_measure(&qargs_ids, &paulis);
            }
            _ => {}
        }
    }
    

    mir_creator.get_block()
}

pub struct DagToMirLowering;

impl StaticPass<DAGCircuit, MIRBlock> for DagToMirLowering {
    fn run(&self, ir: Box<DAGCircuit>, _context: &mut PassContext) -> anyhow::Result<Box<MIRBlock>> {
        Ok(Box::new(lower(&ir)))
    }
}
