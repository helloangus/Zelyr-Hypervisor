# P8-W02 Governance Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P8-W02 detailed design](README.md).

## 1. Logical artifact groups and ownership

W02 is process and documentation work; its logical modules are authoritative
artifact groups.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Machine-contract governance document | `docs/machine-types/machine-contract-governance-v0.1.md` | this design, ADR-024/025/040, ADR §18, task book §1/§2/§8, W01 constraint register | the sole normative home of categories, fact classes, routes, host-independence, compatibility, and freeze rules; it carries no Guest-visible value and no implementation design |
| Documentation routing | one row in `docs/README.md` routing table; one truthful status row in `docs/stages/p8/implementation/README.md` per that index's conventions | governance document location and status | discoverability and truthful status; they do not restate policy |
| Implementation record | `docs/stages/p8/implementation/p8-w02-machine-contract-governance-record.md` (created when work starts) | actual decisions taken | decisions, changed artifacts, deviations; no command logs |
| Verification record | `docs/stages/p8/verification/p8-w02-machine-contract-governance-verification.md` (created when evidence exists) | actual review output | run/not-run evidence per the validation matrix; not part of the design |

The future v1 specification (`rusthv-arm-virt-v1-spec-v0.1.md`) and the
family contracts (boot contract, DTB contract) are *governed* artifacts: this
document defines what they must contain and what gate they must pass, and is
the arbitration reference for them. It does not contain their content.

## 2. Normative inputs to carry forward

The governance document must carry, with citations, the decisions that bound
every machine statement:

- **ADR-024** — a versioned Generic ARM64 VM machine, independent of the host
  SoC; guests never see a "virtual RK3566".
- **ADR-025** — DTB-first Guest description; ACPI is later work.
- **ADR-022** — Guest kernel runs at EL1; no virtual EL2 is exposed.
- **ADR-040** — independent versioning of schema, machine, and management
  ABI; the machine identity is `machine_version`, not a schema or management
  version.
- **ADR-008** — firmware baseline: Non-secure EL2 start under existing
  firmware; the hypervisor handles/proxies PSCI/SMC without owning EL3.
- **ADR §18 open items** — the v1 IPA map, GIC/PCI windows, and virtio slot
  count are recorded as open with their `ADR Required` status.
- **Task book §1/§2/§8** — the category enumeration, Reserved-item
  prohibition, and route labels.
- **W01 constraint register** — the platform-independence, guest-untrusted,
  and versioned-ABI constraints, by reference to the
  [W01 record](../p8-w01-entry-contract-reconciliation-record.md) (future
  path; the governance cites the W01 design until the record exists).

## 3. Machine identity rules

- The machine name is `rusthv-arm-virt-v1` (authority: ADR-024, task book §1).
  The document states the name's authority and that renaming is an
  architecture change, not an edit.
- `machine_version` is expressed by the trailing version token of the
  identity string. Within v1, Guest-visible behavior is immutable per the
  compatibility rules (§8); a v2 is a new identity with its own review, not a
  mutation of v1.
- The document distinguishes the machine identity from any boot-contract or
  DTB-contract version: family contracts carry their own versions and bind to
  a machine identity explicitly (see §8.3).

## 4. Required review categories

Every `rusthv-arm-virt-v1` specification must cover all ten categories. The
governance document defines each category's required content and its
"undecided" state; the parenthetical routing below is the §6 route for the
currently open items.

| # | Category | Required content of a conforming specification | Currently open items and route |
|---|---|---|---|
| C1 | Identity | machine name; machine_version semantics; conformance statement (what claiming v1 means) | freeze authority: `ADR Required` |
| C2 | CPU | Guest EL; vCPU count range and minimum; topology expression; CPU feature baseline and ID-register posture | feature baseline, topology expression: `Specification Investigation` |
| C3 | Guest physical-address categories | RAM category (base/size granularity and supported sizes); reserved-memory categories; MMIO window categories; permanent vs provisionable regions | concrete IPA values, window boundaries: `ADR Required` |
| C4 | Interrupts | GIC generation and interface version; interrupt-number category assignment (SGI/PPI/SPI ranges); maintenance interrupt; what is permanently reserved | concrete interrupt numbers: `ADR Required`; semantics detail: `Specification Investigation` |
| C5 | Timer | architected-timer presentation (counters, virtual timer, frequency expression); EL1 access posture | detail: `Specification Investigation` |
| C6 | Firmware/DTB | PSCI version and mandatory function subset; conduit; DTB provenance rule (Guest-only DTB; which contract defines required nodes — by reference to the DTB contract) | PSCI mandatory subset: `Specification Investigation`; DTB node values: route to W04 contract, then C-category review |
| C7 | Console | the non-Virtio console device category and its placement category | device detail and placement: `Specification Investigation`, values `ADR Required` |
| C8 | Permanent ABI | the list of statements declared immutable within v1 and the mechanism for declaring future immutability | which statements are permanent: reviewed with v1, `ADR Required` for the declaration rule |
| C9 | Reserved space | bounded, documented reservations (e.g., future virtio-MMIO and PCI capacity) with category and review rule — never values, protocols, or formats | reservation values: `ADR Required` |
| C10 | Compatibility and change | compatible-change list; incompatible-change rule (forces a new machine identity); deprecation and reservation-consumption rules | declaration authority: `ADR Required` (same route as freeze authority) |

A specification missing a category, or answering a category with an unrouted
open item, fails the freeze gate (§9).

## 5. Fact classes

Every statement in a machine-contract family document carries exactly one
class:

| Class | Definition | Examples (illustrative, not values) | Change rules |
|---|---|---|---|
| **Permanent Guest-visible fact** | a statement a Guest or image may rely on for as long as the machine identity exists | "the Guest runs at EL1"; "the console is non-Virtio MMIO-mapped" (category level) | immutable within v1; change forces a new identity (§8) |
| **Implementation fact** | a statement about how the hypervisor realizes the contract, not observable as a contract term | allocator behavior behind RAM delivery; internal telemetry | may change freely with review, provided no Guest-visible fact moves |
| **Reservation** | a bounded, documented allowance for future capacity, with no protocol, value, format, or policy defined | "capacity for N future MMIO device windows is reserved" (N itself being a routed value) | consumed only through the C10 rules; a reservation never defines the reserved thing (task book §2 Reserved) |

The class of a borderline statement (for example, whether a DTB node name is
Guest-visible) defaults to *Permanent Guest-visible fact* — the conservative
reading — until the C-category review assigns otherwise.

## 6. Decision-route table

The governance document contains the standing route table; the current rows
are:

| Open item (task book §8) | Route | Route owner | Status handling |
|---|---|---|---|
| Concrete v1 IPA values, GIC/PCI windows, virtio slot count, freeze authority | `ADR Required / Specification Investigation` | ADR process | preserve ADR §18; decision required before specification publication or implementation |
| CPU feature baseline, topology expression, PSCI mandatory subset, timer/console/device details | `Specification Investigation` | the consuming P8 detailed design, using AArch64/Linux/PSCI/GIC sources; never inferred from QEMU | proposals land in the design; values enter the specification only after C-category review |
| DTB construction, image placement/loading, system-register handling, internal compatibility-test mechanics | `Implementation Choice` | approved detailed design (W03–W05, W14, W16) | decide in design while preserving approved contract facts |
| QEMU-versus-hardware observations; later RK3566 behavior | `Platform Investigation` | the observing package's records | environment-specific evidence only; never a Core or machine-ABI rule |
| Predecessor evidence absence or contradictory architecture/security/machine input | `Architecture Change Request / ADR Required` | stage coordinator + ADR process | block the affected choice; record without repairing upstream scope |

Adding, resolving, or re-routing a row is a reviewed governance-document
change. A design that needs a value whose row is unresolved must stop at the
route, not proceed with a plausible default.

## 7. Host-independence rule

A machine-contract family document may not contain, as a Guest-visible
statement or a conformance requirement: a QEMU or RK3566 (or any board/SoC)
name, a host physical address, a host IRQ number, a host firmware property,
or any statement justified only by "QEMU does this". QEMU `virt` may appear
only in test-environment context, clearly separated from contract text. The
acceptance form of the rule is the P8-V03 negative review: the reviewer must
be able to classify every Guest-visible statement as derivable from AArch64
architecture sources, Linux/PSCI/GIC consumption requirements, or routed
Zelyr decisions — never from host observation alone.

## 8. Compatibility and version rules

1. **Compatible changes within v1** (no new identity, reviewed edit):
   corrections that change no Guest-visible statement; implementation-fact
   changes; clarifications; reservation documentation that adds no reserved
   definition.
2. **Incompatible changes** — any move, resize, reassignment, or semantic
   change of a Permanent Guest-visible fact — require a new machine identity
   (`…-v2`), a new specification under the same governance, and an explicit
   compatibility statement for v1 consumers. Silently redefining v1 is an
   architecture violation (ADR change rules).
3. **Family-contract binding.** The boot contract and Guest DTB contract
   (W03/W04) are versioned family contracts; each binds explicitly to a
   machine identity, and each carries its own version and compatibility
   section. A family-contract change that alters a Guest-visible boot or DTB
   fact is treated as an incompatible machine change under rule 2.
4. **Never serialize raw Rust struct memory as a contract.** Any
   machine-contract statement about a data representation must specify
   representation, width, endianness, padding, and version explicitly
   (Coding-Guidelines ABI rule), with compatibility behavior stated; a
   Rust type layout is never the contract.

## 9. Freeze gate

A `rusthv-arm-virt-v1` specification may be published as frozen only when all
of the following hold; the governance document states this gate verbatim:

1. every category (§4) is present and answered with values whose routes (§6)
   have resolved, including an accepted ADR for every `ADR Required` item;
2. every statement carries a fact class (§5) and the C8 permanent-ABI list is
   explicit;
3. the host-independence review (§7) has passed for the document;
4. the consumer review has passed: W03–W09 representatives (their designs or
   records) confirm the values satisfy their contracts' assumptions;
5. the compatibility statement (§8 rules 2–3) is present, and the freeze
   authority recorded by its routed ADR decision has approved.

Publication before gate completion is a scope violation this design prohibits.
W02's own completion (P8-V02/V03) requires the *route and gate* to be
complete — it does not require, and must not claim, the gate to be passed.

## 10. Consumer rules

Each consuming package: designs against categories and gates, not values;
routes every value it needs through §6; binds its family contract to a machine
identity with its own version (§8.3); and respects the class assignment and
reservation prohibition. A consumer that believes a category or route is
wrong raises a reviewed governance change, not a local exception.
