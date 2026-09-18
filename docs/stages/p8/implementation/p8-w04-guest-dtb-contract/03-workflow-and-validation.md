# P8-W04 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W04 detailed design](README.md).

## 1. Preconditions and failure boundary

Before editing, the implementer verifies it has loaded the documents named in
the parent README and confirms the observable state: no Guest DTB contract or
builder exists; the W02 governance document (or the W02 design as interim)
and the W03 boot contract (or the W03 design as interim) are the available
category/fact sources; the W06–W09 contracts are planned or in flight in
parallel — their facts are cited by plan/design path, never assumed beyond
their published scope.

Stop and obtain direction instead of guessing when any of the following
occurs:

- writing a fact appears to require choosing a compatible string, address,
  interrupt specifier, PSCI function ID, register frame, or topology form —
  mark the value routed per the W02 route table; do not select it;
- a consumer contract (W06–W09) and this catalog disagree about who owns a
  DTB fact — the DTB contract owns *presentation*; the consumer owns the
  mechanism behind it. If the disagreement is about ownership itself, stop
  and route it; do not silently re-split the boundary;
- a needed fact has no published binding — apply
  [the binding-source rule](01-dtb-fact-catalog.md) §13 (drop or route), do
  not invent a Zelyr binding;
- the contract text starts to specify builder code, a schema file, or a DTB
  binary — that is out of W04 scope entirely; stop and restate as a
  requirement.

## 2. Ordered implementation steps

### Step 1 — verify the assumed inputs

Target: working context; DTB contract §1 (Identity and binding).

Work: confirm the W02 categories (governance document or design) and W03
boot facts (contract or design) are available; record the citation status
and the planned/in-flight status of W06–W09 in the implementation record
(`../p8-w04-guest-dtb-contract-record.md`, created in this step).

**Acceptance:** every source the catalog will cite is identified with its
status.  
**Failure/blocker:** a source conflict is routed per the task book §8
handling, not reworded locally.

### Step 2 — draft the Guest-DTB contract

Target: `docs/machine-types/guest-dtb-contract-v0.1.md`.

Work: write the document with the status header and the eight sections fixed
in [the fact catalog](01-dtb-fact-catalog.md) §1, instantiating the wire
format/generation rules (§2 there), fact groups D1–D10, the binding-source
rule, and the compatibility section. Every fact carries its source; every
routed value carries its route.

**Acceptance:** all sections present; D1–D10 stated with sources; the
wire-format section pins the Devicetree-specification binding and forbids
native-struct serialization; no value selected.  
**Failure/blocker:** a fact without an authoritative source is dropped or
routed — never written unsourced.

### Step 3 — consistency review

Target: the draft document; verification record.

Work: run the C-1–C-11 checks of
[the consistency review](02-consistency-and-host-leakage-review.md) §2
against the draft in their pre-freeze form (categories and routes). Fix the
contract where a check cannot be stated or a source is misidentified.

**Acceptance:** every check evaluable against the draft; every fact
source-traceable.  
**Failure/blocker:** a failing check caused by an upstream category gap is
routed to W02's route table, not papered over.

### Step 4 — host-leakage review

Target: the draft document; verification record.

Work: run the L-1–L-7 checks of
[the leakage review](02-consistency-and-host-leakage-review.md) §3 against
the draft, including the accidental-channel check over examples, comments,
and bootargs phrasing. Record the review outcome.

**Acceptance:** no statement fails L-1–L-7; each prohibition's authority
basis is recorded in the contract.  
**Failure/blocker:** a failing statement is removed or rewritten as a
category/route.

### Step 5 — Linux-consumption review

Target: the draft document.

Work: for each fact group, confirm the presentation form comes from a
published binding ([the binding-source rule](01-dtb-fact-catalog.md) §13)
and that a stock Linux of the pinned fixture series could consume the
presented facts without Zelyr-specific patches. Confirm Host semantics were
not imported: no check or fact derives its *content* from host observation.

**Acceptance:** every group names its binding source; no private binding;
consumption requires no Zelyr patch.  
**Failure/blocker:** a fact that would require a private binding is dropped
or routed per §1.

### Step 6 — wire discovery and closure review

Targets: `docs/README.md` routing row;
`docs/stages/p8/implementation/README.md` status row (per that index's
conventions, coordinated with parallel W06–W20 work); verification record.

Work: add the routing row and truthful status row; run the validation
matrix; confirm the handoff checklist and the plan's acceptance wording
("Guest-only, machine-consistent DTB review and host-leakage criteria. No
DTB output is claimed.").

**Acceptance:** one-link reachability; truthful status; DV01–DV08 recorded.  
**Failure/blocker:** failing items are recorded as failed with diagnosis; no
criterion is weakened to pass.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W04-DV01 → P8-V05 | input and boundary review | inspect contract §1–§2 against W02/W03 and the P2 boundary plans | machine/boot binding stated; generation-source rule excludes Host DTB/PlatformInfo as inputs; P2 boundary cited as prohibition | the contract rests on stated authority; not that any builder exists |
| W04-DV02 → P8-V05 | fact completeness review | map fact groups D1–D10 to the plan scope (CPU/topology, memory, chosen, PSCI, timer, GIC, console, compatible/model, reserved memory, minimal devices) | every scope element covered; v1 device-set exclusion explicit (D10) | the Guest-visible fact set is enumerated; not that a DTB exists |
| W04-DV03 → P8-V05 | source-authority review | per fact, verify exactly one named source | every fact sourced (category, boot fact, consumer contract, published binding, or routed value) | no unsourced or invented fact; not that sources are frozen |
| W04-DV04 → P8-V05 | consistency review | run C-1–C-11 in pre-freeze form against the draft | every check evaluable; every fact traceable; set-equality checks stated for memory/PSCI/console | DTB facts are machine-consistent by construction rule; not that generated DTBs comply yet |
| W04-DV05 → P8-V06 | host-leakage review | run L-1–L-7 against the draft, including comment/bootargs channels | no statement leaks; each prohibition carries its authority basis | the leakage criteria exist and the contract satisfies them; not that future generated DTBs comply (W14/W16 automate that) |
| W04-DV06 → P8-V05 | binding/consumption review | per group, name the published binding; assess stock-Linux consumability | every group binding-sourced; no Zelyr-private binding; no host semantics imported | Linux can consume the presented facts as specified; not that a kernel boot is proven |
| W04-DV07 → P8-V06 | output-exclusion scan | search W04 artifacts for builder code, schema files, DTB binaries, node values | none present; routed values appear only as routes | no DTB output or value is claimed; nothing else |
| W04-DV08 → W04 closure | claim hygiene review | search all W04 artifacts for freeze/completion/output language | none present | scope discipline; nothing else |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with method, input, date, and reason. P8-V05 and P8-V06 are satisfied
only when DV01–DV08 are recorded. No validation here generates a DTB, boots
Linux, proves P8-V04 or any other P8 validation, or substitutes for the W02
freeze gate.

## 4. Error, security, and observability model

W04 adds no runtime path. Its error model is contractual: the consistency
and leakage checks define the *acceptance* a future DTB generator must meet,
and the generator's own failure behavior (rejecting an unsatisfiable fact)
is a requirement handed to its owning implementation, not designed here.
Security properties: the DTB is the primary Guest-visible description of the
machine, so it is an information-boundary artifact — L-1–L-7 keep host
topology, addresses, IRQs, and firmware facts out of Guest reach (ADR-007,
ADR-024); the generation-source rule prevents accidental disclosure through
reuse of Host data; the binding-source rule prevents unreviewed ABI surface.
Observability: both reviews are stated as automated-checkable properties so
W14/W16 can consume them as drift and leak detectors; W04's own evidence
lives in the verification record with planned/run/blocked/failed kept
distinct.

## 5. Handoff checklist

Before handing W04 to a reviewer, provide:

- the exact changed-file list (expected: the contract, routing row, status
  row, implementation record, verification record);
- DV01–DV08 evidence paths and run status, including explicit not-run entries;
- the pinned Devicetree-specification revision and binding sources recorded
  in the contract (or the recorded blocker if unresolvable);
- confirmation that no node value, compatible string, address, interrupt
  specifier, PSCI function ID, register frame, or topology value was fixed;
  that no builder, schema, or DTB binary was created; and that no Host DTB
  or PlatformInfo fact was used as input;
- the open items handed onward: every routed value the contract defers, for
  the W02 route table; and the source-authority map for W06–W10, W14–W16
  consumers; and
- the recorded conflicts or boundary disagreements with W06–W09 contracts,
  if any, with their routes.
