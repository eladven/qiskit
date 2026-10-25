use qiskit_passmanager::PassManager;
use qiskit_qpy::native_load_qpy;
use qiskit_circuit::dag_circuit::DAGCircuit;
use qiskit_mir::lowering::pass_manager::DagToMirLowering;
use qiskit_mir::mir::Mir;


#[test]
fn lower_bell_pbc(){
    let dump = std::fs::read("tests/fixtures/bell_pbc.qpy").unwrap();
    let circuits = native_load_qpy(&dump).unwrap();
    let circuit_data = &circuits[0];
    let dag = DAGCircuit::from_circuit_data(circuit_data, false, None, None).unwrap();

    let mut pm = PassManager::new();
    pm.try_push_static_pass(DagToMirLowering).unwrap();
    pm.run::<DAGCircuit, Mir>(dag).unwrap();
}