use std::collections::{BTreeMap, BTreeSet};

use crate::{
    instruction::MirInstruction,
    mir::Mir,
    ops::{
        MirOp,
        pbc::{AngleType, Pauli, PauliList, PauliRotation},
    },
    qid::{Qid, QidInfo, QidList},
    symbol_table::SymbolIdList,
};
use qiskit_circuit::dag_circuit::DAGCircuit;
use qiskit_circuit::operations::OperationRef;
use qiskit_passmanager::{PassContext, StaticPass};

fn pauli_from_bits(x: bool, z: bool) -> Pauli {
    match (x, z) {
        (false, false) => Pauli::I,
        (true, false) => Pauli::X,
        (false, true) => Pauli::Z,
        (true, true) => Pauli::Y,
    }
}

pub fn lower(dag: &DAGCircuit) -> Mir {
    let mut mir = Mir::new();
    let mut qubits: BTreeSet<u32> = BTreeSet::new();
    let mut qid_for_qubits: BTreeMap<u32, Qid> = BTreeMap::new();

    for node in dag.topological_op_nodes(false) {
        let instr = dag.dag()[node].unwrap_operation();
        for q in dag.get_qargs(instr.qubits) {
            qubits.insert(q.0);
        }
    }

    for qubit in qubits {
        // TODO: remove print when MIR has real printer
        println!("create_qubit {qubit}");
        let qid = mir
            .resource_table_mut()
            .add_qid(QidInfo::new(crate::qid::BitLayout::Base(1)))
            .expect("qid allocation should succeed");
        qid_for_qubits.insert(qubit, qid);
    }

    for node in dag.topological_op_nodes(false) {
        let instr = dag.dag()[node].unwrap_operation();
        let qargs = dag.get_qargs(instr.qubits);
        let qids = qargs.iter().map(|q| qid_for_qubits[&q.0]).collect();
        match instr.op.view() {
            OperationRef::PauliProductRotation(rotation) => {
                let paulis = rotation
                    .x
                    .iter()
                    .zip(rotation.z.iter())
                    .map(|(&x, &z)| pauli_from_bits(x, z))
                    .collect();
                let angle = rotation.angle.try_float().unwrap_or(0.0);
                // TODO: remove print when MIR has real printer
                println!("pauli_rotate {:?} {:?} {:?}", qids, paulis, angle);
                let op = MirOp::PauliRotation(PauliRotation::new(
                    PauliList::new(paulis),
                    AngleType::new(angle),
                ));
                mir.instructions_mut().push_back(MirInstruction::new(
                    op,
                    QidList::new(qids),
                    SymbolIdList::default(),
                    SymbolIdList::default(),
                ));
            }
            OperationRef::PauliProductMeasurement(measurment) => {
                let paulis: Vec<Pauli> = measurment
                    .x
                    .iter()
                    .zip(measurment.z.iter())
                    .map(|(&x, &z)| pauli_from_bits(x, z))
                    .collect();
                // TODO: remove print when MIR has real printer, and implemented pauli measure
                println!("pauli_measure {:?} {:?}", qids, paulis);
            }
            _ => {}
        }
    }
    mir
}

pub struct DagToMirLowering;

impl StaticPass<DAGCircuit, Mir> for DagToMirLowering {
    fn run(&self, ir: Box<DAGCircuit>, _context: &mut PassContext) -> anyhow::Result<Box<Mir>> {
        Ok(Box::new(lower(&ir)))
    }
}
