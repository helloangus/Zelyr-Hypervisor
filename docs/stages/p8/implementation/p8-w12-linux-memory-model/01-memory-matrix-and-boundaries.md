# P8-W12 Memory Matrix and Boundary Contract

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W12 detailed design](README.md).

## 1. Upstream contracts consumed and their failure boundaries

"Evidenced" means present in the P2/P4 stage implementation and verification
material referenced by their handoff plans (P2-W10
`../../../p2/plans/p2-w10-p3-p4-handoff-contract.md`, P4-W09
`../../../p4/plans/p4-w09-closeout-p5-handoff.md`); planned wording alone never
satisfies a row.

| Contract | Owning plan | Assumed, evidenced fact | Failure boundary if different |
|---|---|---|---|
| Stage-2 capability boundary | P4-W02 (`../../../p4/plans/p4-w02-stage2-address-space.md`) | Independent Guest address-space lifecycle with map/unmap/protect/query/activation and current-path invalidation | `P4DependencyIssue`; the whole matrix is blocked |
| Fault classification and context | P4-W06 (`../../../p4/plans/p4-w06-fault-isolation-diagnostics.md`) | Stage-2 translation/permission faults distinguishable with Guest/vCPU/PC/state/IPA/access-type/mapping/reason context; Guest faults never become unexplained EL2 panics | Diagnostic rows degrade to Linux-observable outcomes only; `P4DependencyIssue` |
| Protected/Host-owned range exclusion | P4-W06/W09 | Selected Hypervisor-owned ranges are blocked from Guest access | Host-isolation rows blocked; `P4DependencyIssue`; never simulated in W12 |
| Ownership ledger and allocation | P2-W10 (`../../../p2/plans/p2-w10-p3-p4-handoff-contract.md`) | Per-page ownership with allocation ownership metadata; reserved ranges never allocated | Ownership-review rows blocked; `P2DependencyIssue` |
| Approved machine map facts | P8-W02/W04 (sibling designs `../p8-w02-machine-contract-governance/README.md`, `../p8-w04-guest-dtb-contract/README.md`) | Guest IPA categories (RAM window, reserved regions, console MMIO window, reservations) approved and reflected in the Guest DTB | Matrix parameters unrecordable; W12-DV04/DV05 blocked |
| Classified CPU/MMU behavior | P8-W05 (`../p8-w05-linux-cpu-virtualization/README.md`) | MMU/cache/TLB/barrier behavior classification for Linux (Direct/Emulate/Reject/Hidden/Unsupported) | Categorization of Linux-internal TLB/barrier evidence adjusts to the delivered classification; `P8IntegrationGap` if missing |
| Linux capability | P8-W09/W10 (`../p8-w09-virtual-console-single-cpu-linux/README.md`, `../p8-w10-linux-smp-bringup/README.md`) | Linux boots to shell (1 vCPU) and SMP (2/4 vCPU) | Workload rows blocked; prerequisite missing |
| Fixture memory workloads | P8-W15 (`../p8-w15-reproducible-linux-fixture/README.md`) | `mem-exercise`/`fork-storm` per 02 §5 present with stable markers | Affected rows blocked; integration gap against W15 |

## 2. Evidence model: two layers

**Layer L1 — Linux-internal memory behavior.** Linux's own allocators, Guest
page tables, TLB maintenance, COW, and kernel/user switching are Guest-EL1
activities invisible to Stage-2 except as (non-)events. Evidence is:

- the declared workload completes with its markers (02 §5);
- no Stage-2 fault occurs during L1 windows, except faults that the P4-W06
  classification and the P8-W05 classification both explain as classified,
  expected behavior (recorded, never dismissed);
- after L1 windows, ownership queries still show the Guest RAM backing
  single-owned and bounded (01 §6).

L1 proves Linux memory management *operates over* Stage-2. It does not prove
anything about Stage-2 internals.

**Layer L2 — Guest-map boundary behavior.** Guest accesses near or beyond the
approved map's edges have explicit expected outcomes (01 §5). Evidence is the
observed Stage-2 event, its W13-classified context, and the contained VM
outcome. L2 proves the map boundary and isolation hold under real Linux.

## 3. Evidence categories (matrix content)

| ID | Category | Layer | Scenario reference |
|---|---|---|---|
| M12-1 | RAM initialization: Linux sees exactly the declared RAM window and DTB memory facts (W04) | L1 | boot on each class |
| M12-2 | Allocator/page-table operation: Linux boot-time and runtime allocation, Guest page-table builds, TLB maintenance complete without unclassified Stage-2 events | L1 | each class, after boot |
| M12-3 | COW and process lifecycle: `fork-storm` completes; kernel/user switching across declared workload | L1 | `fork-storm` |
| M12-4 | RAM-size classes: small/normal/larger each boot and run the declared workload subset | L1+L2 | 01 §4 |
| M12-5 | Unmapped-IPA boundary: access beyond RAM window faults with classified context | L2 | 01 §5 N1 |
| M12-6 | Reserved-region boundary: access into declared reserved region faults or behaves per approved contract; never silent | L2 | N2 |
| M12-7 | Host-owned boundary: access toward Host/hypervisor-owned ranges is excluded (P4-W06 basis) | L2 | N3 |
| M12-8 | MMIO window boundary: in-window console access works (W09); out-of-window access faults without affecting the console | L2 | N4 |
| M12-9 | Permission boundary (conditional): if the approved map declares Guest-visible permission distinctions, crossing them faults with permission classification | L2 | N5 |
| M12-10 | Ownership preservation: single-owner, donation-bounded, protected-range exclusion holds during and after all runs including shutdown | both | 01 §6 |

## 4. RAM classes (no capacities selected)

| Class | Definition (relational, capacity recorded at implementation) | Required runs |
|---|---|---|
| `small` | The minimum RAM class that boots the pinned Linux fixture to an interactive shell within the approved map's RAM window; recorded capacity must be justified by a boot success at that value and a failure or non-claim below it | boot + `mem-exercise` (small parameter) + N1/N4 probes |
| `normal` | The baseline class the boot contract and W09/W10 evidence used; the default for W16's matrix | full L1 set + all L2 probes |
| `larger` | Above `normal`, within the approved map's RAM window; justified by exercising more Guest page-table/allocator structure; never beyond the window | boot + `mem-exercise` (larger parameter) + N1 probe |

Rules: capacities are recorded in the implementation record with their map
bounds and justification; a class above the approved window is invalid, not a
finding; class values are validation parameters, not machine ABI facts, and do
not enter the [W14](../p8-w14-machine-abi-compatibility/README.md) matrix.

## 5. Boundary and negative cases

Every case declares: the Guest action, the expected Stage-2/VM outcome, the
required diagnostic context (W13 minimum-context contract), and the contained
scope. A case that cannot name its expectation from the approved map facts is
`blocked`, not guessed.

| ID | Guest action | Expected outcome | Containment |
|---|---|---|---|
| N1 | Read/write/execute at an IPA beyond the RAM window (probe at a declared offset past the end, and at a declared hole inside the map window) | Stage-2 translation fault; classified with IPA, access type, syndrome | VM-scoped diagnostic; Linux observes its own abort/panic; EL2 unaffected |
| N2 | Access a declared reserved region inside the Guest view | Stage-2 fault or the approved contract's declared behavior; never silent success | as N1 |
| N3 | Attempt access whose IPA translates toward a Host-owned/protected range (as exposed by the map) | Blocked per evidenced P4-W06 protection; fault classified | as N1; additionally ownership query shows no grant occurred |
| N4 | Access the first address outside the declared console MMIO window | Stage-2 fault; console (in-window) continues to work before and after | console unaffected; VM-scoped diagnostic |
| N5 | Conditional: access crossing an approved Guest-visible permission distinction | Stage-2 permission fault, classified as permission (not translation) | as N1 |
| N6 | Clean shutdown (W03/W06 path) after all classes/probes | Ownership ledger shows no leaked or dual-owned page; address space teardown complete | — |

Negative-case probes are Guest-side attempts performed by the fixture's probe
program (02 §5); they are ordinary Linux loads that dereference declared
addresses, not new EL2 fault-injection APIs. If inducing a case requires an
EL2-side injection mechanism that does not exist, the case is blocked with a
`P4DependencyIssue`/`P8IntegrationGap` decision, not implemented here.

## 6. Ownership and Guest-untrusted invariants reviewed

The step-6 review (workflow) checks, per run and cumulatively:

- **Single ownership:** every Guest RAM backing page has exactly one owner in
  the evidenced ledger during and after runs; no page is simultaneously
  Host-reserved and Guest-mapped (ADR §19 `MUST`).
- **Donation bounds:** the Guest mapping is a subset of the donated backing for
  the whole run; no map-time or run-time growth beyond it (ownership before
  mapping).
- **Protected-range exclusion:** N3-class behavior matches the evidenced P4
  exclusion; no probe ever succeeds against a Host-owned range.
- **Guest-untrusted handling:** every observed Linux-caused fault has a
  VM-facing classification and contained outcome; zero occurrences of an EL2
  panic, hang, or cross-VM effect attributable to a Linux fault.
- **No host leakage:** Stage-2 diagnostics and Guest-visible structures contain
  no Host physical addresses (W04 host-leak criteria apply to diagnostics too).

A violated invariant of the ADR §19 class is a `HypervisorInvariantViolation`:
hypervisor-level failure per P0-W14
(`../../../p0/plans/p0-w14-panic-failure-classification.md`), never absorbed as
a Guest fault, and never "fixed" by a W12 edit.
