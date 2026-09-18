# P6-W10 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P6-W10 detailed design](README.md).  
**Companions:**
[01-interrupt-semantics-contract.md](01-interrupt-semantics-contract.md)
(semantic authority; record schema §1).

## 1. Preconditions and failure boundary

Before any step, the implementer verifies the document set from the parent
README and inspects the tree (`git ls-files`; P0 scaffold — no Rust sources,
no W07–W09 implementation records yet). Prerequisites are assumed contracts
from plans and handoffs
([P4-W09](../../../p4/plans/p4-w09-closeout-p5-handoff.md),
[P5-W10](../../../p5/plans/p5-w10-closeout-p6-handoff.md)).

Stop and record a blocker (never patch a prerequisite) when:

- a W05/W06/W07/W08/W09 design or implementation contradicts a rule of
  [01-interrupt-semantics-contract.md](01-interrupt-semantics-contract.md)
  §2–§5 — raise an Architecture Change Request against the owning design;
  W10 reconciles semantics, it does not redefine mechanisms;
- the W08 design presents no priority-mask control and a scenario needs L2 —
  record L2 unavailable (contract §2.1); do not simulate it;
- exercising a rule appears to require a P7 run-state concept or a P8 vGIC
  semantic — out of scope; stop;
- the P6-V13–P6-V15 evidence cannot be produced because W11's suite does not
  exist yet — that is an ordering dependency, recorded as blocked evidence,
  not a semantics failure.

## 2. Ordered implementation steps

### Step 1 — prerequisite reconciliation

Target: implementation record (`../p6-w10-interrupt-semantics-record.md`,
created in this step).

Work: read the W05/W06 (timer), W07 (lifecycle/queue), W08 (presentation),
and W09 (maintenance) designs and records; verdict per cross-boundary rule
(contract §5, §2 L2, §4 S-PD4): confirmed, deviant, or absent. Confirm the
W07 state surface can carry the L3 `enabled` semantics without a new
interface.

**Acceptance:** verdict per prerequisite recorded with evidence locations.  
**Failure/blocker:** a deviate verdict on a load-bearing rule blocks step 3;
record the conflict per §1.

### Step 2 — realize the L3 `enabled` semantics in the W07 surface

Target: the W07-owned vIRQ lifecycle implementation (semantic increment
only).

Work: where W07's implemented contract exposes the enable/disable state
surface, integrate the §2.1 L3 rules: disabled events accumulate per §4,
occupy no LR capacity, and admit on enable; masking bookkeeping never
suppresses Host records (C-M1). No new public API; no queue-discipline
change; validation per the P5 boundary on the control path.

**Acceptance:** code review maps each L3 behavior to a contract rule; the
P5-authorized control path is the only Guest-reachable trigger.  
**Failure/blocker:** if realization would require changing W07's queue
discipline or adding a public interface — stop; that is a W07 design change,
not a W10 step.

### Step 3 — integrate with deferred delivery and entry/exit

Target: W06/W08/W09 integration points only.

Work: confirm S-C4/S-C5 against the implemented deferred-expiry and
save/restore behavior; confirm that no rule above assigned any scheduler
responsibility (review checklist: grep the semantics for run-state, wake,
preempt concepts — none may appear as dependencies).

**Acceptance:** each S-C rule has an implemented-behavior verdict; the
scheduler-responsibility review finds none.  
**Failure/blocker:** a contradiction is a step-1-class conflict; stop and
record.

### Step 4 — create the Interrupt Semantics v0 record

Target: `docs/stages/p6/implementation/p6-interrupt-semantics-v0.md`
(created in this step, factual content only).

Work: per the artifact schema of
[01-interrupt-semantics-contract.md](01-interrupt-semantics-contract.md) §1:
lifecycle term meanings as implemented (citing W07/W08/W09), masking layers
and availability, band relation, pending/repeated policy, timer-plus-vIRQ
rules, maintenance-completion reference, the §8 non-freeze statement, and
known limits with evidence links. No planned-but-unimplemented behavior may
be stated as fact; the L2-unavailable case (if any) is stated.

**Acceptance:** every factual statement carries an evidence link or an
explicit limitation; the non-freeze statement is present.  
**Failure/blocker:** a statement without evidence is removed or marked as a
limitation, never softened into an implication.

### Step 5 — boundary review

Target: review findings in the implementation record.

Work: review Guest-untrusted controls (contract §6) against the implemented
validation paths; review non-loss rules (S-M1, S-PD3, S-PD4) against W09-DV
and W11 scenario coverage; confirm no machine-ABI statement exists anywhere
in W10 artifacts.

**Acceptance:** each §6 requirement maps to an implemented check or a
recorded W12 case; the P8-exclusion review is explicit.  
**Failure/blocker:** an unreviewed control path blocks closure.

### Step 6 — evidence cross-citation and handoff

Target: `../../verification/p6-w10-interrupt-semantics-verification.md`
(created when evidence exists).

Work: cite (never duplicate) the W11 suite evidence for SEM-MASK, SEM-PRIO,
SEM-MULTI, SEM-REPEAT and the Host-side telemetry correlation; record
run/failed/blocked/not-run per validation row below; run the handoff
checklist (§5). P6-V13–P6-V15 wording carries its proof boundary: passing
does not prove full GIC specification coverage.

## 3. Validation matrix

Evidence destinations: W11 suite runs land in
`../../verification/p6-w11-validation-guest-interrupt-suite-verification.md`;
W10's cross-citation and review evidence lands in
`../../verification/p6-w10-interrupt-semantics-verification.md`.

| ID | Test or review | Technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W10-DV01 → P6-V13 | mask/pending/unmask (SEM-MASK) | W11 VG-IRQ-03 run with L1 masking; repeat with L3 when available | IRQ pending while masked; observable after unmask; exactly one completion; no false completion of another event | S-M1 and L1/L3 behavior in QEMU; not L2 unless presented; not real-hardware timing |
| W10-DV02 → P6-V14 | repeated events (SEM-REPEAT) | W11 VG-IRQ-04 run: repeats while pending, then while active | documented S-PD1/S-PD2 policy observed; no state corruption; determinate final count | the bounded-accumulation policy; not GIC-conformance of edge/level semantics generally |
| W10-DV03 → P6-V15 | timer plus vIRQ (SEM-PRIO) | W11 VG-IRQ-05 run with both classes ready | both progress; band-A-first relation observed where simultaneous; no starvation in the entry | the documented band relation; not full priority specification coverage |
| W10-DV04 → P6-V13/V15 | different-priority survival | pending band-A behind presented band-B; complete band-B | band-A presented next per S-P2; not lost, not reordered into a completed state | relation survival in pending state; not preemption of in-progress presentation |
| W10-DV05 | multiple pending (SEM-MULTI, supports P6-V12/V13) | W11 VG-IRQ-02 run | distinct events complete independently (S-PD3) | distinct-event independence; not unbounded queue behavior (W07/W09 own capacity) |
| W10-DV06 | semantics-across-exit review | inspect S-C5 evidence from W06/W08 exit/re-entry runs | each exit class has an implemented-behavior verdict in the v0 record | semantics preservation as evidenced; not new behavior beyond those contracts |
| W10-DV07 | Guest-untrusted control review | exercise malformed/unauthorized enable/mask controls via the P5 boundary (with W12) | rejected or contained as GuestFault-class; no Host state change | containment of Guest controls; not production DoS resistance (W12 boundary) |
| W10-DV08 | record factualness review | audit the v0 record against step 4 acceptance | no unevidenced factual claim; non-freeze statement present | documentation integrity (feeds P6-V27); not stage closure |

## 4. Error, security, and observability model

Errors on this semantic surface follow the containment classes of the W12
boundary: Guest-caused anomalies are GuestFault-class and VM-local; Host
bookkeeping anomalies follow the W09 E1–E4 model; neither may masquerade as
the other (the S-M1 rule is also a security rule — it makes fabricated
completions impossible by construction). Observability: masking, priority
band outcomes, pending accumulation, and repeats are visible through the
W07–W09 telemetry event sets under the P0-W13 namespace
([P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md),
[P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md));
W10 adds no new event mechanism, only required coverage requirements for
W13's P6-V24 map (band decisions observable in Host telemetry and Guest
records). QEMU success does not prove real-hardware semantics; the v0 record
must state its environment.

## 5. Handoff checklist

Before handing W10 to a reviewer, provide:

- the exact changed-file list, including the v0 record path;
- step 1 prerequisite verdicts and any L2-unavailable recording;
- confirmation that no new public API, queue-discipline change, scheduler
  concept, vGIC MMIO semantic, or machine-ABI statement was introduced;
- W10-DV01–DV08 status (passed/failed/blocked/not-run) with evidence paths,
  including blocked entries where W11's suite had not yet run;
- the v0 record's evidence-link completeness audit;
- open items for W11 (scenario assertions frozen to contract §7), W12
  (Guest-control cases), W13 (P6-DOC-02 composition, P6-V24 coverage), P7
  (wakeup-relevant semantics), and P8 (non-freeze statement) — without
  resolving their contracts here;
- any recorded Architecture Change Request or Specification Investigation
  (numeric priority encoding) with its owner.
