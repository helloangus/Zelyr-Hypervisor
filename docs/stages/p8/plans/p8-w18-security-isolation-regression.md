# P8-W18 — Security and isolation regression

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Plan regression proving Linux integration retains the Guest-untrusted containment boundary.

## Scope

Cover source P8.18: illegal IPA/MMIO/sysreg/PSCI/topology, Guest crash, infinite loop, and interrupt storm; require protection of Hypervisor, Host, other VM, and other vCPU recoverability.

## Out of scope

New capability policy, device assignment, IOMMU isolation, global recovery design, or declaring all Guest errors harmless.

## Work sequence

1. Inspect W05–W13, W16, and P5 security facts.
2. Define each malicious or abnormal scenario and controlled expected result.
3. Relate failures to VM/vCPU diagnostics and scheduler/interrupt containment.
4. Define cross-VM and Host-protection observations.
5. Review unresolved failure classes as security or architecture blocks.

## Acceptance and closure

P8-V24 requires declared containment evidence for every listed scenario; a Linux panic is never equivalent to Hypervisor panic.

## Handoff

W20 and P9+ receive factual containment limits only after evidence exists.
