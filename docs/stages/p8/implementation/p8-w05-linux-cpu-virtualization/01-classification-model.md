# P8-W05 Classification Model

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P8-W05 detailed design](README.md).

## 1. Purpose and authority

This file defines the Guest CPU behavior classification: the class
vocabulary, the rules for assigning a class, the per-area inventory for the
Linux Guest, and the invariants the classification preserves. The vocabulary
is plan-authorized; the per-area rows are design proposals whose concrete
values (feature baseline, presentation sets, per-register detail) are routed
`Specification Investigation` items per the task book §8 and reviewed into
the W02 machine categories — a row's *class* is posture this design owns; a
row's *values* are machine facts nobody may fix here.

## 2. Class definitions

Every Guest-originated CPU operation (register access, instruction,
feature-dependent path) resolves to exactly one class:

| Class | Meaning | Hypervisor action on the normal path | Guest-visible result |
|---|---|---|---|
| **Direct** | The operation executes on hardware under the configured virtualization controls and the architecturally defined behavior is correct for the Guest | None — no trap, or trap with pure return | The architecture's own behavior |
| **Emulate** | The operation traps to EL2 and the hypervisor must supply the architecturally correct effect | A registered domain handler performs the effect; the Guest resumes | The architecture's own behavior, produced by the handler |
| **Reject** | The operation is not provided to the Guest; attempted use must fail visibly | The controlled-rejection path ([03](03-controlled-failure-and-diagnostics.md) §2): architecturally legal Guest fault + structured diagnostic + containment + telemetry | A Guest-observable fault; never silent wrong behavior, never a hypervisor crash |
| **Hidden** | The feature is not provided and is *concealed at discovery* so correct software does not attempt it | Discovery-time shaping (e.g., ID-register presentation) by the feature-posture handler; an attempt anyway re-resolves per its trap classification (normally Reject) | The feature appears absent |
| **Unsupported** | The feature/operation is known to be unimplemented and is not concealed; encounters are expected and must land in the Reject path with full diagnostics | As Reject, with the classification recorded as a known gap in the diagnostic | As Reject |

Assignment rules:

1. **Architectural correctness test first.** If the hardware, under the
   configured virtualization controls (Guest EL1, Stage-2 active per the P4
   contracts), gives the architecturally correct result with no hypervisor
   action, the class is Direct.
2. **Provided-effect test second.** If a trap is architecturally required or
   inevitable and a hypervisor effect is part of the machine's approved
   behavior (timer, interrupt controller interface, PSCI, feature
   presentation), the class is Emulate and names its owning domain route.
3. **Provision test third.** If the machine does not provide the operation:
   - concealable at discovery and concealment approved → Hidden;
   - not concealed (or concealment not approved) → Unsupported when the gap
     is known and documented, Reject as the catch-all for anything
     unresolved (rule 4).
4. **Fail-closed default.** An operation that cannot be classified —
   unknown encoding, registry miss, missing handler, handler failure — is
   Reject. There is no class whose outcome is "continue as if nothing
   happened".
5. **Class ≠ value.** A class never freezes a feature bit, register value,
   or encoding; those stay routed. Two configurations of the same machine
   version differ in routed values, never in class semantics.

## 3. State and ownership of the classification

The classification is a read-only-after-initialization registry owned by the
CPU-virtualization posture (one registry per hypervisor image, not per VM;
per-VM variation enters only through configuration the machine categories
approve, such as vCPU count). Domain packages register handlers against
routes at initialization; after Guest execution begins, the registry and its
routes are immutable. The registry is not Guest-writable and contains no
Guest addresses. This ownership rule is what makes the decision point
auditable: exactly one code path resolves classes, and its data cannot
change underneath a running Guest.

## 4. Per-area classification inventory

The areas are the plan's scope list. Rows marked *(proposed)* carry the
review caveat of §1; routed values are marked *(routed)*.

### 4.1 Guest exception level

| Behavior | Class | Basis |
|---|---|---|
| Guest execution at EL1 | Direct (posture fact) | ADR-022; machine identity |
| Attempted access to EL2 registers from EL1 | Reject per register group *(proposed; per-group detail routed)* | No virtual EL2 (ADR-022); the traps exist architecturally |
| Attempted EL3 / secure-monitor operations from EL1 | Reject | ADR-008 (no EL3 ownership); Guest never sees secure state |

### 4.2 System registers

| Behavior | Class | Basis |
|---|---|---|
| EL1 control/status and address-translation registers (SCTLR_EL1, TCR_EL1, TTBR0/1_EL1, MAIR_EL1, VBAR_EL1, context-dependent EL1 state) | Direct | Hardware applies them under Stage-2; P4 Stage-2 assumed contract |
| ID/feature registers, EL1 reads | Emulate — feature-posture handler presents the approved baseline *(presentation values routed, W02 C2)* | Hidden-class posture at discovery (decision 5 of the README); Linux probes these before using features |
| Timer registers (CNTV/CNTP/CNTPCT and controls) | Emulate — routed to the [P8-W08](../p8-w08-linux-timer-integration/README.md) timer contract | Task book work map; P6 timer assumed contract |
| Interrupt-controller system registers (GICv3 ICC_*_EL1 interface) | Emulate — routed to the [P8-W07](../p8-w07-linux-vgicv3/README.md) vGIC contract | Task book work map; P6 interrupt assumed contract |
| PSCI invocation via the architecture conduit (HVC/SMC as the approved conduit defines) | Emulate — routed to the [P8-W06](../p8-w06-psci-virtualization/README.md) PSCI contract | Task book work map; conduit respects the P5 hypercall contract |
| Unallocated / reserved encodings; implementation-defined registers with no approved meaning | Reject *(proposed; list maintained as Specification Investigation)* | Fail-closed default |

### 4.3 MMU / TLB

| Behavior | Class | Basis |
|---|---|---|
| Guest stage-1 enablement, translation-table writes, attr changes | Direct | Stage-2 composes with Guest stage-1 per ADR-018; P4 assumed contract |
| Guest TLBI affecting the Guest's own stage-1 / current VM context | Direct *(proposed: hardware VMID/Stage-2 scoping confines the effect to the VM; the isolation obligation sits on the P4/Stage-2 side, not here)* | Inter-VM isolation invariant (ADR §19); P4 TLB facts |
| Hypervisor-side invalidation obligations after Host-side changes to Guest mappings | Implementation fact — owned by the P4 Stage-2 contract; not a Guest-visible class | ADR §19 cross-CPU invalidation MUST |

### 4.4 Cache maintenance

| Behavior | Class | Basis |
|---|---|---|
| Guest-issued DC/IC maintenance (by VA, set/way, point of coherency/unification) | Direct | Architectural behavior is correct under Stage-2 for Guest-owned lines |
| Hypervisor cache-maintenance obligations when Host code writes Guest-executable memory | Implementation fact — owned by the loader/integration path; recorded here so it is not mistaken for a Guest-visible class | Coherency requirements of the pinned boot protocol (W03 §2) |

### 4.5 Barriers

| Behavior | Class | Basis |
|---|---|---|
| DSB / DMB / ISB | Direct | Architectural semantics are correct for the Guest |

### 4.6 WFI / WFE

| Behavior | Class | Basis |
|---|---|---|
| WFI | Emulate — block the vCPU on the P7 scheduler block/wakeup contract; wake on approved events (virtual interrupt, timer deadline, notification) per the P7/P6 contracts | P7-W06 block/wakeup assumed contract; ADR §5 (scheduler-visible WFI) |
| WFE | Direct *(proposed, with the Reserved escalation trigger stated in the README decision 7)* | No correctness need to trap when events are coherent; overcommit policy is out of scope |

### 4.7 Timer

Covered in §4.2's timer row: the area is Emulate with the mechanism wholly
owned by [P8-W08](../p8-w08-linux-timer-integration/README.md); this design
claims only that timer attempts must never fall through the decision point
unhandled.

### 4.8 Interrupt masking

| Behavior | Class | Basis |
|---|---|---|
| PSTATE DAIF masking/unmasking | Direct | Hardware; pending semantics per the P6 interrupt-semantics assumed contract (masking does not imply completion) |

### 4.9 CPU features

| Behavior | Class | Basis |
|---|---|---|
| Feature baseline presented to the Guest | Hidden/Emulate posture — discovery shaped; baseline values routed *(W02 C2 `Specification Investigation`)* | Machine identity; host independence (never "what QEMU presents") |
| Attempted use of a concealed feature | Reject (per its trap classification) | Fail-closed default |
| Topology-dependent behavior (SMP enumeration, affinity hints) | Direct at the CPU level; topology values routed (W02 C2; D2 of the [DTB contract](../p8-w04-guest-dtb-contract/README.md)) | Presentation is DTB/ID-posture, not trap behavior |

## 5. Classification invariants

1. Every trapped Guest operation resolves to exactly one class, by rule 4
   if necessary. Totality is testable (see
   [05-validation-and-handoff.md](05-validation-and-handoff.md) V08 rows).
2. Direct costs zero hypervisor decisions on its normal path; Emulate and
   Reject do bounded, schedulable work per the exit-path bounds of the
   established contracts (no unbounded work, no blocking in exit context).
3. Reject outcomes are VM-scoped: they never raise a global hypervisor
   failure (ADR §19) and never mutate another VM's state.
4. Hidden presentation and Emulate effects are idempotent per operation and
   side-effect-accurate to the architecture — a handler that cannot produce
   the architectural effect fails into Reject, not into approximation.
5. The registry is immutable during Guest execution (§3); any runtime
   reclassification is a design-change event, not a runtime state change.
6. No class, row, or value encodes a host fact: features are presented per
   routed machine decisions, never per host observation (ADR-024/052).
