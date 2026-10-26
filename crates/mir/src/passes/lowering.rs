use std::{
    assert_eq,
    collections::{BTreeMap, BTreeSet},
    format,
};

use crate::{
    instruction::MirInstruction,
    mir::Mir,
    ops::{
        MirOp,
        pbc::{AngleType, Pauli, PauliList, PauliMeasure, PauliRotation},
    },
    qid::{Qid, QidInfo, QidList},
    symbol_table::{Symbol, SymbolId, SymbolIdList, SymbolType},
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
    let mut clbits: BTreeSet<u32> = BTreeSet::new();
    let mut qid_for_qubits: BTreeMap<u32, Qid> = BTreeMap::new();
    let mut sym_for_clbits: BTreeMap<u32, SymbolId> = BTreeMap::new();

    for node in dag.topological_op_nodes(false) {
        let instr = dag.dag()[node].unwrap_operation();
        for q in dag.get_qargs(instr.qubits) {
            qubits.insert(q.0);
        }
        for c in dag.get_cargs(instr.clbits) {
            clbits.insert(c.0);
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

    for clbit in clbits {
        println!("create_clbit {clbit}");
        let sym = mir
            .symbol_table_mut()
            .add(Symbol::new(format!("c{clbit}"), SymbolType::Bool))
            .expect("symbol allocation should succeed");
        sym_for_clbits.insert(clbit, sym);
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
                let node_clbits = dag.get_cargs(instr.clbits);
                assert_eq!(
                    node_clbits.len(),
                    1,
                    "PauliProductMeasurement should have 1 clbits"
                );
                let sym_id = sym_for_clbits[&node_clbits[0].0];
                // TODO: remove print when MIR has real printer
                println!("pauli_measure {:?} {:?} -> {:?}", qids, paulis, sym_id);
                let op = MirOp::PauliMeasure(PauliMeasure::new(PauliList::new(paulis)));
                mir.instructions_mut().push_back(MirInstruction::new(
                    op,
                    QidList::new(qids),
                    SymbolIdList::default(),
                    SymbolIdList::new(vec![sym_id]),
                ));
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
