# P7 work-package plans

**Status:** Planning index. Plans are bounded work packages; implementation and evidence are not claimed.

Read the architecture baseline, [P7 task book](../task-book-v0.1.md), this index, and the selected plan. Read the Coding Guide and an approved detailed design before code. Put implementation traceability in [../implementation/](../implementation/) and verification evidence in [../verification/](../verification/).

| Plan | Prerequisites | Primary consumers |
|---|---|---|
| [W01](p7-w01-entry-contract-reconciliation.md) | ADR, task book, P0–P6 handoffs | W02–W14 |
| [W02](p7-w02-scheduler-admission-lifecycle.md) | W01 | W03–W11 |
| [W03](p7-w03-placement-configuration.md) | W02 | W05, W07–W08, W11 |
| [W04](p7-w04-preemption-context-switch.md) | W02, P6 timer/event contract | W05, W07, W09–W10 |
| [W05](p7-w05-shared-mn-multivm.md) | W03, W04 | W08, W10–W11 |
| [W06](p7-w06-block-wakeup.md) | W02, P6 event contract | W08–W11 |
| [W07](p7-w07-pause-stop-fault.md) | W02–W04 | W08–W11 |
| [W08](p7-w08-smp-reschedule-idle.md) | W03, W05–W07, P3 notification | W11 |
| [W09](p7-w09-accounting-diagnostics.md) | W02, W04, W06–W07 | W11, W13–W14 |
| [W10](p7-w10-validation-guest-suite.md) | W04–W07 | W11 |
| [W11](p7-w11-stress-invariants.md) | W08–W10 | W12 |
| [W12](p7-w12-qemu-regression.md) | W11 | W14 |
| [W13](p7-w13-performance-baseline.md) | W09 | W14 |
| [W14](p7-w14-documentation-p8-handoff.md) | W12–W13 | P8 |

The dependency map in the task book is authoritative. A listed consumer may rely only on evidenced predecessors.
