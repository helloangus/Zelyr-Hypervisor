//! P1-W09 boot-phase tracker, ordered sequencer and terminal failure routing.
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use super::{console, fatal};
use crate::arch::aarch64::{baseline, capabilities, exceptions, stage1};

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
static REPLAYED: AtomicBool = AtomicBool::new(false);

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

fn emit_marker(phase: InitPhase, event: console::MarkerEvent) {
    let line = console::format_marker(phase.label(), event);
    console::transport_line(line.as_str());
}

/// All earlier events are derivable from the monotone tracker word. The
/// channel exists only after the Console mechanism returns successfully.
fn materialize_replay() {
    if TRACKER.position() != LifecyclePosition::Entered(InitPhase::Console)
        || !console::channel_available()
        || REPLAYED.swap(true, Ordering::Relaxed)
    {
        fail_phase(
            InitPhase::Console,
            FailureReason::new("marker replay invariant"),
        );
    }
    for phase in InitPhase::ALL.into_iter().take(5) {
        emit_marker(phase, console::MarkerEvent::Enter);
        emit_marker(phase, console::MarkerEvent::Complete);
    }
    emit_marker(InitPhase::Console, console::MarkerEvent::Enter);
}

fn transition_failure(phase: InitPhase, error: SequenceError) -> ! {
    let _ = error.current; // The tracker remains authoritative for the report.
    fail_phase(phase, FailureReason::new("lifecycle transition invariant"))
}

pub(crate) fn phase_enter(phase: InitPhase) {
    if let Err(error) = TRACKER.advance_enter(phase) {
        transition_failure(phase, error);
    }
    // Console.enter is materialized by replay after the channel exists.
    if matches!(phase, InitPhase::FatalPath | InitPhase::Stage1) {
        emit_marker(phase, console::MarkerEvent::Enter);
    }
}

pub(crate) fn phase_complete(phase: InitPhase) {
    if let Err(error) = TRACKER.advance_complete(phase) {
        transition_failure(phase, error);
    }
    if matches!(
        phase,
        InitPhase::Console | InitPhase::FatalPath | InitPhase::Stage1
    ) {
        emit_marker(phase, console::MarkerEvent::Complete);
    }
}

/// The W02 glue calls this only after the sequencer returns from Stage1.
pub(crate) fn enter_stable() {
    if let Err(error) = TRACKER.mark_stable() {
        transition_failure(InitPhase::Stable, error);
    }
    console::transport_line("ZELYR P1 STABLE");
}

pub(crate) fn fail_phase(phase: InitPhase, reason: FailureReason) -> ! {
    match phase {
        InitPhase::Entry => panic!("entry rejection escaped W01 boundary: {}", reason.message()),
        InitPhase::Runtime
        | InitPhase::Capabilities
        | InitPhase::El2Baseline
        | InitPhase::Exceptions
        | InitPhase::Console => panic!("phase={} reason={}", phase.label(), reason.message()),
        InitPhase::FatalPath => {
            if fatal::fatal_path_ready() {
                fatal::report_fatal_phase(phase, reason)
            } else {
                fatal::readiness_failure()
            }
        }
        InitPhase::Stage1 | InitPhase::Stable => fatal::report_fatal_phase(phase, reason),
    }
}

fn capabilities_step() {
    capabilities::build_capability_report();
}
fn el2_baseline_step() {
    baseline::establish_el2_baseline();
}
fn exceptions_step() {
    exceptions::install_el2_exception_entry();
}
fn console_step() {
    if let Err(error) = console::bring_up_early_console() {
        let reason = match error {
            console::ConsoleError::EnableReadbackMismatch => "console enable readback mismatch",
        };
        fail_phase(InitPhase::Console, FailureReason::new(reason));
    }
    materialize_replay();
    capabilities::render_report(&mut console::transport_line);
}
fn fatal_path_step() {
    fatal::arm_fatal_path();
}
fn stage1_step() {
    if let Err(error) = stage1::enable_host_stage1() {
        fail_phase(
            InitPhase::Stage1,
            FailureReason::new(error.static_message()),
        );
    }
}

pub(crate) fn run_init_sequence() {
    phase_enter(InitPhase::Capabilities);
    capabilities_step();
    phase_complete(InitPhase::Capabilities);

    phase_enter(InitPhase::El2Baseline);
    el2_baseline_step();
    phase_complete(InitPhase::El2Baseline);

    phase_enter(InitPhase::Exceptions);
    exceptions_step();
    phase_complete(InitPhase::Exceptions);

    phase_enter(InitPhase::Console);
    console_step();
    phase_complete(InitPhase::Console);

    phase_enter(InitPhase::FatalPath);
    fatal_path_step();
    phase_complete(InitPhase::FatalPath);

    phase_enter(InitPhase::Stage1);
    stage1_step();
    phase_complete(InitPhase::Stage1);
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
