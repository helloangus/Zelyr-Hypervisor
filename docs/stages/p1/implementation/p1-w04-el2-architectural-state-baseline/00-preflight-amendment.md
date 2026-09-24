# W04 preflight design correction

**Status:** Proposed detailed design; implementation not claimed.
**Scope:** Control-effect synchronization.
**Version:** v0.2
**Owner/change context:** P1 preflight, 2026-09-24.
**Supersedes:** Architecture §7 and control-write §2 note (c).

## Baseline and required foundation

At a05fca6, W01/W02 entry and panic code exist, but no W04 mechanism exists.
A declared known baseline requires control effects visible before dependent
instructions. Required: the synchronization below. Reserved: SMP synchronization.
Out of scope: Guest execution and new timer policy.

## Corrected boundary

An ordinary MSR/MRS pair is not a context synchronization event. The internal
write_sysreg boundary must issue ISB after each control write and before its
readback or dependent instruction. In particular, HCR_EL2's non-VHE posture
must take effect before lower-EL register accesses. Final declaration follows
all synchronized writes and masked verification.

No new state, allocation, lock, public interface or failure fallback is added.
ISB belongs to the audited write boundary. Existing readback mismatch takes
the terminal baseline failure route. Register masks and category ownership
are unchanged.

Arm's [memory ordering explanation](https://developer.arm.com/community/arm-community-blogs/b/architectures-and-processors-blog/posts/memory-access-ordering-part-3---memory-access-ordering-in-the-arm-architecture)
describes the ISB context synchronization role. Register encodings remain
pinned to the Arm revision in the implementation record.

## Acceptance

W04-DV02/DV04/DV05 inspect emitted ordering, especially HCR to EL1 accesses
and the final declaration. Results belong in W04's separate verification record.
This amendment and a successful build do not themselves establish P1-V07.

