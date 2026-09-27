# P2-W03 runtime evidence custody

**Status:** Recorded reference-QEMU evidence, 2026-09-27.
**Scope:** Twelve W03 cases and four upstream smoke cases; no hardware or allocator claim.

See the [verification report](../p2-w03-runtime-verification.md) for commands,
artifact identities, outcomes and proof limits. The JSON summaries and serial
logs here retain compact results from the final runs. The two stack-review
files rerun the final conservative four-callback audit against the same ELFs;
the original run reports used three callbacks. Both bounds fit the unchanged
64 KiB stack with the required 8 KiB reserve. Runtime traces are unchanged.

`manifest.json` records SHA-256 and size for local raw artifacts, including
compressed instruction traces, fixture inputs and disassemblies. Those large
files remain at their recorded repository-relative `target/` paths on this
machine; this is not an off-host archive. Preserve them before cleaning target.
