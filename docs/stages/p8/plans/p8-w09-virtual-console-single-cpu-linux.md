# P8-W09 — Virtual console and single-vCPU Linux

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Plan the non-Virtio console and complete one-vCPU Linux path to interactive initramfs userspace.

## Scope

Cover source P8.8–P8.9: early/kernel/userspace console, shell interaction, serial containment, complete boot milestones, retained boot log, and clean baseline path.

## Out of scope

Virtio console, final device register model, userspace distribution, performance tuning, or SMP validation.

## Work sequence

1. Inspect W03–W05, W07–W08, and fixture inputs.
2. Define the complete observable one-vCPU boot and console acceptance path.
3. Relate console ownership and malformed access to isolation requirements.
4. Define boot-log markers and the userspace-shell completion criterion.
5. Review that start_kernel alone cannot close the package.

## Acceptance and closure

P8-V12 and P8-V13 require shell-reaching console evidence and retained boot markers, with containment conditions. They are not current results.

## Handoff

W10, W16, and W19 receive the single-CPU baseline and expected observables.
