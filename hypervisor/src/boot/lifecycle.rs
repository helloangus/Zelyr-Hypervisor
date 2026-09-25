//! P1-W09's pure boot-phase vocabulary and single-word tracker foundation.
//! The sequencer and marker emission are deliberately not linked here yet.
use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InitPhase {
    Entry,
    Runtime,
    Capabilities,
    El2Baseline,
    Exceptions,
    Console,
    FatalPath,
    Stage1,
    Stable,
}
impl InitPhase {
    pub(crate) const ALL: [Self; 8] = [
        Self::Entry,
        Self::Runtime,
        Self::Capabilities,
        Self::El2Baseline,
        Self::Exceptions,
        Self::Console,
        Self::FatalPath,
        Self::Stage1,
    ];
    /// Stable is a terminal outcome, not a production phase.
    pub(crate) const fn sequence_index(self) -> Option<usize> {
        match self {
            Self::Entry => Some(1),
            Self::Runtime => Some(2),
            Self::Capabilities => Some(3),
            Self::El2Baseline => Some(4),
            Self::Exceptions => Some(5),
            Self::Console => Some(6),
            Self::FatalPath => Some(7),
            Self::Stage1 => Some(8),
            Self::Stable => None,
        }
    }
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Entry => "entry",
            Self::Runtime => "runtime",
            Self::Capabilities => "capabilities",
            Self::El2Baseline => "el2-baseline",
            Self::Exceptions => "exceptions",
            Self::Console => "console",
            Self::FatalPath => "fatal-path",
            Self::Stage1 => "stage1",
            Self::Stable => "stable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LifecyclePosition {
    PreBoot,
    Entered(InitPhase),
    Completed(InitPhase),
    Stable,
    Unknown(usize),
}
impl LifecyclePosition {
    const fn decode(raw: usize) -> Self {
        if raw == 0 {
            return Self::PreBoot;
        }
        if raw == 17 {
            return Self::Stable;
        }
        if raw <= 16 {
            let index = raw.div_ceil(2);
            let phase = InitPhase::ALL[index - 1];
            return if raw & 1 == 1 {
                Self::Entered(phase)
            } else {
                Self::Completed(phase)
            };
        }
        Self::Unknown(raw)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SequenceError {
    pub(crate) current: LifecyclePosition,
}

pub(crate) struct BootPhaseTracker {
    encoded: AtomicUsize,
}
impl BootPhaseTracker {
    pub(crate) const fn new() -> Self {
        Self {
            encoded: AtomicUsize::new(0),
        }
    }

    /// Wait-free snapshot for exception/fatal context. P1 has one masked boot
    /// CPU; this load publishes no other object and needs no cross-CPU order.
    pub(crate) fn position(&self) -> LifecyclePosition {
        LifecyclePosition::decode(self.encoded.load(Ordering::Relaxed))
    }

    pub(crate) fn advance_enter(&self, phase: InitPhase) -> Result<(), SequenceError> {
        let Some(index) = phase.sequence_index() else {
            return Err(SequenceError {
                current: self.position(),
            });
        };
        self.compare(2 * (index - 1), 2 * index - 1)
    }

    pub(crate) fn advance_complete(&self, phase: InitPhase) -> Result<(), SequenceError> {
        let Some(index) = phase.sequence_index() else {
            return Err(SequenceError {
                current: self.position(),
            });
        };
        self.compare(2 * index - 1, 2 * index)
    }

    pub(crate) fn mark_stable(&self) -> Result<(), SequenceError> {
        self.compare(16, 17)
    }

    fn compare(&self, expected: usize, next: usize) -> Result<(), SequenceError> {
        // One boot writer; compare_exchange still rejects duplicate/out-of-
        // order calls without mutating state. Readers need only the word.
        match self
            .encoded
            .compare_exchange(expected, next, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => Ok(()),
            Err(raw) => Err(SequenceError {
                current: LifecyclePosition::decode(raw),
            }),
        }
    }
}
pub(crate) static TRACKER: BootPhaseTracker = BootPhaseTracker::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FailureReason(&'static str);
impl FailureReason {
    pub(crate) const fn new(message: &'static str) -> Self {
        Self(message)
    }
    pub(crate) const fn message(self) -> &'static str {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_phase_encodings_are_exact_and_monotone() {
        let tracker = BootPhaseTracker::new();
        assert_eq!(tracker.position(), LifecyclePosition::PreBoot);
        for (offset, phase) in InitPhase::ALL.iter().copied().enumerate() {
            assert_eq!(phase.sequence_index(), Some(offset + 1));
            assert_eq!(tracker.advance_enter(phase), Ok(()));
            assert_eq!(tracker.position(), LifecyclePosition::Entered(phase));
            assert_eq!(tracker.advance_complete(phase), Ok(()));
            assert_eq!(tracker.position(), LifecyclePosition::Completed(phase));
        }
        assert_eq!(tracker.mark_stable(), Ok(()));
        assert_eq!(tracker.position(), LifecyclePosition::Stable);
    }

    #[test]
    fn duplicates_and_out_of_order_calls_preserve_state() {
        let tracker = BootPhaseTracker::new();
        assert_eq!(
            tracker.advance_complete(InitPhase::Entry),
            Err(SequenceError {
                current: LifecyclePosition::PreBoot
            })
        );
        assert_eq!(
            tracker.advance_enter(InitPhase::Runtime),
            Err(SequenceError {
                current: LifecyclePosition::PreBoot
            })
        );
        assert_eq!(tracker.advance_enter(InitPhase::Entry), Ok(()));
        assert_eq!(
            tracker.advance_enter(InitPhase::Entry),
            Err(SequenceError {
                current: LifecyclePosition::Entered(InitPhase::Entry)
            })
        );
        assert_eq!(
            tracker.advance_enter(InitPhase::Stable),
            Err(SequenceError {
                current: LifecyclePosition::Entered(InitPhase::Entry)
            })
        );
        assert_eq!(
            tracker.position(),
            LifecyclePosition::Entered(InitPhase::Entry)
        );
    }

    #[test]
    fn unknown_decode_preserves_the_full_word() {
        assert_eq!(
            LifecyclePosition::decode(usize::MAX),
            LifecyclePosition::Unknown(usize::MAX)
        );
        assert_eq!(
            LifecyclePosition::decode(18),
            LifecyclePosition::Unknown(18)
        );
        assert_eq!(InitPhase::Stable.sequence_index(), None);
        assert_eq!(InitPhase::Stable.label(), "stable");
        assert_eq!(FailureReason::new("sample").message(), "sample");
        assert_eq!(TRACKER.position(), LifecyclePosition::PreBoot);
    }
}
