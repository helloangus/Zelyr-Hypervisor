# P8 work-package plans

**Status:** Planning index. Plans are bounded work packages; implementation,
ABI freezing, and evidence are not claimed.

Read the Architecture baseline, [P8 task book](../task-book-v0.1.md), this
index, and the selected plan. Read the Coding Guide and an approved detailed
design before code. Put implementation traceability in
[../implementation/](../implementation/) and verification evidence in
[../verification/](../verification/).

| Plan | Prerequisites | Primary consumers |
|---|---|---|
| [W01](p8-w01-entry-contract-reconciliation.md) | ADR; P0–P7 factual handoffs | W02–W20 |
| [W02](p8-w02-machine-contract-governance.md) | W01; ADR/ABI/machine-model review | W03–W20; P9+ |
| [W03](p8-w03-linux-boot-contract.md) | W01–W02 | W04, W09–W10, W15–W16 |
| [W04](p8-w04-guest-dtb-contract.md) | W02–W03 | W09–W10, W14, W16 |
| [W05](p8-w05-linux-cpu-virtualization.md) | W01–W03; P4–P7 execution facts | W09–W10, W13, W18 |
| [W06](p8-w06-psci-virtualization.md) | W02–W05 | W10, W14, W16, W18 |
| [W07](p8-w07-linux-vgicv3.md) | W01–W02; P6 interrupt facts | W09–W10, W16, W18 |
| [W08](p8-w08-linux-timer-integration.md) | W01–W02; P6–P7 timer facts | W09–W10, W16–W18 |
| [W09](p8-w09-virtual-console-single-cpu-linux.md) | W03–W05, W07–W08 | W10, W16, W19 |
| [W10](p8-w10-linux-smp-bringup.md) | W04–W09 | W11–W20 |
| [W11](p8-w11-scheduler-linux-integration.md) | W10; P7 scheduler facts | W16–W18 |
| [W12](p8-w12-linux-memory-model.md) | W02–W05, W10 | W16, W18 |
| [W13](p8-w13-guest-fault-diagnostics.md) | W05–W10 | W16, W18, W20 |
| [W14](p8-w14-machine-abi-compatibility.md) | W02, W04, W06–W09 | W16, W20; P9+ |
| [W15](p8-w15-reproducible-linux-fixture.md) | W03–W04 | W09–W10, W16–W19 |
| [W16](p8-w16-automated-linux-regression.md) | W09–W15 | W17–W20; P9 |
| [W17](p8-w17-linux-performance-baseline.md) | W10–W12, W16 | W20; P9+ |
| [W18](p8-w18-security-isolation-regression.md) | W05–W13, W16 | W20; P9+ |
| [W19](p8-w19-validation-guest-dual-track.md) | W01, W09–W10, W15–W16 | W20; P9+ |
| [W20](p8-w20-documentation-closure-handoff.md) | W14, W16–W19 | P9 |

The task book dependency map is authoritative. A consumer may rely only on
evidenced predecessor facts; planned documents are not such evidence.
