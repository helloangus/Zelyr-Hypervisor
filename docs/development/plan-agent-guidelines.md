# Zelyr Plan Agent Guidelines — Concise Mandatory Guide

**Status:** Mandatory for every Plan Agent task.  
**Scope:** Turn an approved stage task book into an implementable design; do
not write production code or redefine project architecture.

Read this document, the repository [agent instructions](../../AGENTS.md), the
baseline ADR, and the applicable Stage Task Book before planning. Read only the
relevant sections of the [detailed reference](plan-agent-design-guidelines-v0.1.md)
when the routing table below requires it.

## Authority and scope

For planning, authority is:

```text
Architecture ADR > current Stage Task Book > frozen ABI / machine model
> established module contract > stage-local design freedom
```

On conflict, label the issue `Architecture Change Request` or `ADR Required`;
do not silently redesign a higher-level decision.

Classify every requirement as **Required**, **Reserved** (must not block a
future design, but is not implemented now), or **Out of Scope**. Do not bring
future-stage functionality forward or turn a temporary implementation into a
permanent contract.

## Required plan content

Each stage plan must provide, at the appropriate level of detail:

1. Goal, ADR constraints, scope classification, existing dependencies, and
   requirement-to-validation traceability.
2. Subsystems and modules: responsibilities, non-responsibilities, inputs,
   outputs, owned state, dependencies, lifecycle, and failure boundary.
3. Core objects and ownership: typed identities/addresses, owner of every
   mutable state transition, lifetime and destruction/revocation semantics.
4. Interfaces and function contracts: preconditions, postconditions, errors,
   side effects, allocation/blocking/interrupt-context rules, and ABI status.
5. Explicit lifecycle/state machines, initialization order, runtime data/control
   flow, failure rollback, and shutdown/resource-reclamation paths.
6. Concurrency/synchronization, security boundaries, telemetry, platform
   differences, validation plan, exit-criterion mapping, extension points, and
   explicitly classified open questions.

Do not use booleans as hidden state machines, registries as implicit owners, or
large `Manager`/global objects as a substitute for boundaries.

## Architecture guardrails

- Separate mechanism from policy. EL2 exposes secure resource mechanisms;
  management and scheduling policy do not leak into generic core APIs.
- Preserve layers: Core is architecture- and board-independent; Arch contains
  ISA mechanisms; SoC/Board/BSP/Quirk code remains outside Core. Use platform
  capabilities, never board-name branches, for behavior selection.
- Keep Host platform and Guest virtual-machine models separate. Physical CPU
  and vCPU are separate objects; static binding is a scheduler policy.
- Model memory ownership before mapping. Guest address space, memory object,
  region, DMA ownership, and IOMMU state need explicit compatible lifecycles.
- Treat Guest, device, firmware, and management-domain input as untrusted.
  Guest-caused faults are recoverable VM-facing errors unless an invariant says
  otherwise.
- Capability/handle + rights + generation is the authorization model. Roles,
  fixed VM IDs, and Control Domain identity are not equivalent authority.
- EL2 does not absorb complex configuration parsing, authentication secrets, or
  general filesystems. Version public management/ABI/machine contracts.
- Keep device frontend/backend roles separable; virtio descriptors and shared
  memory are security and consistency boundaries. Device assignment is a
  transaction with rollback.
- Telemetry is a designed interface, not an accumulation of debug prints.

## Validation and handoff

Specify unit, host, QEMU, Validation Guest, Linux Guest, security/property/fuzz,
and real-hardware validation as applicable. State what each proves; QEMU does
not define hardware semantics or prove real-hardware correctness.

Before handoff, check architecture consistency, scope, explicit ownership and
state authority, failure/recovery, observability, and every stage exit
criterion. Output the implementation-level design and an open-question list;
do not output source code in place of a plan.

## When to consult the detailed reference

| Trigger | Detailed-reference sections |
|---|---|
| Module/object/function design | 7–13, 71–77, 84–86 |
| Arch, platform, BSP, CPU, VM, memory | 14–26, 79–83 |
| IRQ, timer, device, virtio, DMA, IOMMU | 27–34, 78 |
| Capability, IPC, Control Domain, configuration, ABI | 35–50, 95–96 |
| Locks, SMP, errors, transactions, lifecycle | 52–63, 89, 93–94 |
| Validation, performance, temporary implementation | 63–70, 97–98 |
| Timing, state authority, telemetry, completion | 87–92, 99–106 |

If a task is not covered by this table, search the detailed reference before
inventing a rule.
