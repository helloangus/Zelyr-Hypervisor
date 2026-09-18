# P1-W11 Scope and Security Review Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W11 detailed design](README.md).

This file defines the P1-V19 review: the checks that the implemented P1 tree
contains no unintended security relaxation, no unintended RWX mapping, a
current unsafe inventory, and no later-stage mechanism. It is a review
procedure over the tree that exists when W11 runs — it does not implement
controls and does not claim their satisfaction.

## 1. Review items

| # | Review | Method | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| S1 | Untrusted-input and range checks | read the W01 entry validation and W03 capability paths against their contracts and the Coding Guidelines' untrusted-input rules | entry parameters and boot-supplied facts are validated (range, alignment, state) before use; no check is bypassed by the W11 trigger code | entry boundary discipline as designed; not hardware behavior of edge values |
| S2 | No unintended RWX | enumerate the W08 mapping classes actually wired; compare against the mapping contract | every mapped region's permission/execution attributes match the contract; no region is writable and executable; vectors are not writable per the W08 contract | mapping hygiene of the P1 image; not future dynamic mappings (P2+) |
| S3 | Unsafe inventory current | walk every `unsafe` block/function in the P1 tree against the P0-W10 unsafe-governance process | each has a nearby `SAFETY` justification; each appears in the inventory process; no unjustified or stale entry | inventory discipline at review time; not future code |
| S4 | Stage boundary intact | diff-scope review of the P1 changes against the task book's out-of-scope list | no Guest, SMP/PSCI, GIC, DTB-discovery/PlatformInfo, allocator/heap, virtio, Control-Domain, or board-name mechanism entered; no board/platform name branches in generic code | P1 stayed in scope; not later-stage design quality |
| S5 | No relaxation on fault paths | read the W09 routes and W11 trigger code | no fault path disables a control another path established (e.g., no permission loosening, no trap disabling) before the terminal behavior | fault paths are containment-neutral; not the diagnostics' completeness (W07's contract) |
| S6 | Trigger containment | build/inspect the default image selection | default build contains no reachable `fault_scenario` reference; scenario builds differ only by the selection and trigger code | normal P1 scope unchanged; not that scenario code itself is fault-free (its failures are recorded evidence) |

## 2. Review workflow

1. Fix the review baseline: the exact source state (commit/image identity)
   under review, recorded in the verification document.
2. Perform S1–S6 in order; each result records the evidence pointer (file,
   line-range, or build artifact) and the outcome (**passed**, **failed**,
   **blocked**, **not run**).
3. A failed item is a finding with an owning package (the contract owner),
   not a local patch inside W11: S1/S2 findings go to W01/W03/W08, S3 to the
   unsafe-governance process, S4/S5 to the stage review, S6 to W11 itself.
4. The review is repeated whenever the P1 tree changes materially before
   stage closure; the verification document records which baseline each
   result refers to.

What the review collectively proves: at the recorded baseline, the P1 tree
shows no unintended relaxation, no unintended RWX, a current unsafe
inventory, and an intact stage boundary — by inspection. What it does not
prove: absence of all vulnerabilities, correctness of hardware behavior, or
properties of code written after the baseline.

## 3. Evidence destination

`../../verification/p1-w11-negative-fault-validation-verification.md`,
scope/security review section: baseline identity, per-item outcomes with
evidence pointers, findings with owners, and explicit not-run entries. The
[P1-W12](../p1-w12-p1-documentation-handoff/README.md) evidence map
references this section for P1-V19.
