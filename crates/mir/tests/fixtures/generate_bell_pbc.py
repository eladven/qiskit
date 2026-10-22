import os
from qiskit import QuantumCircuit, qpy
from qiskit.transpiler import generate_preset_pbc_pass_manager


def make_bell_circuit():
    qc = QuantumCircuit(2)
    qc.h(0)
    qc.cx(0, 1)
    qc.measure_all()
    return qc

if __name__ == "__main__":
    qc = make_bell_circuit()
    pm = generate_preset_pbc_pass_manager()
    qc_pbc = pm.run(qc)
    print(qc_pbc)
    out_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "bell_pbc.qpy")
    with open(out_path, "wb") as file:
        qpy.dump(qc_pbc, file)

