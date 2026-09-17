# P3 work-package plans

**Status:** Planning index. Each linked document is a planned, bounded work
package; none claims implementation or verification.

## Reading order

For a P3 package, read the [architecture baseline](../../../adr/adr-000-architecture-baseline-v0.1.md), the [P3 task book](../task-book-v0.1.md), this index, and the selected plan. Then read the mandatory guide for the activity: the Plan Agent guide for detailed design, or the Coding Guide and an approved detailed design before code changes. Consult prerequisite implementation and verification evidence before starting work.

P0 provides engineering governance, toolchain, diagnostics, and portability rules. P1 provides the EL2 execution baseline. P2 must provide the documented PlatformInfo CPU topology, boot-memory map, and safe allocator/heap contracts before P3 implementation begins. Those predecessor outcomes are planning prerequisites, not assertions about the current repository.

## Dependency and execution map

| Plan | Prerequisites | Primary consumers |
|---|---|---|
| [W01](p3-w01-cpu-topology-inputs.md) | P0–P2 handoff contracts, ADR | W02–W05, W10, P4 |
| [W02](p3-w02-secondary-cpu-bring-up.md) | W01, P1 EL2 boot contract, P2 platform start input | W03–W05, W09, W13 |
| [W03](p3-w03-physical-cpu-lifecycle.md) | W01, W02 | W04–W05, W07–W15, P4 |
| [W04](p3-w04-per-cpu-runtime.md) | W02, W03, P1 exception baseline, P2 allocation contracts | W05–W12, P4 |
| [W05](p3-w05-smp-boot-synchronization.md) | W02–W04 | W06, W10, W12–W15 |
| [W06](p3-w06-concurrency-synchronization.md) | W04, W05, P0 unsafe/diagnostic rules | W07–W10, W12–W15, P4 |
| [W07](p3-w07-cross-cpu-notification.md) | W03, W04, W06 | W08, W11–W15, P4 |
| [W08](p3-w08-tlb-shootdown-transport.md) | W03, W06, W07 | W11–W15, P4 |
| [W09](p3-w09-cpu-local-exception-interrupt.md) | W02–W04, P1 exception baseline | W10–W15, P4 |
| [W10](p3-w10-smp-safety-audit.md) | W01–W09, P0–P2 baseline records | W11–W15, P4 |
| [W11](p3-w11-smp-observability.md) | W03, W07, W08, W10, P0 telemetry governance | W12–W15, P4 |
| [W12](p3-w12-smp-stress-failure-tests.md) | W05–W11 | W13–W15, P4 acceptance |
| [W13](p3-w13-qemu-smp-regression.md) | W02, W05–W12, P0 QEMU/CI contracts | W14–W15, P4 entry review |
| [W14](p3-w14-p4-smp-handoff.md) | W01–W13 | P4 detailed design and implementation |
| [W15](p3-w15-stage-documentation-acceptance.md) | W01–W14 | P3 closure review, P4 handoff |

Dependencies are acyclic: execution proceeds from topology and bring-up through lifecycle/per-CPU state, boot coordination and synchronization, cross-CPU facilities, audit/telemetry/testing, then handoff and closure. Evidence belongs under `../implementation/` and `../verification/`; create it only when the corresponding work is actually performed.
