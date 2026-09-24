//! P1's fixed reference-console transport. W09 owns its phase and markers.
use core::fmt::{self, Write};
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

use super::identity::BUILD_IDENTITY;

/// QEMU virt's first PL011. P2 discovery supersedes this fixed assumption.
pub(crate) const CONSOLE_REGION_BASE: u64 = 0x0900_0000;
#[allow(dead_code)] // W08 consumes the named mapping requirement.
pub(crate) const CONSOLE_REGION_SIZE: u64 = 0x1000;
const DATA: u64 = 0;
const FLAGS: u64 = 0x18;
const CONTROL: u64 = 0x30;
const TX_FULL: u32 = 1 << 5;
const UART_ENABLE: u32 = 1;
const TX_ENABLE: u32 = 1 << 8;
const REQUIRED_CONTROL: u32 = UART_ENABLE | TX_ENABLE;
const LINE_CAPACITY: usize = 128;
const TRUNCATED: &[u8] = b" [truncated]";
static CHANNEL_AVAILABLE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConsoleError {
    EnableReadbackMismatch,
}

/// W09 owns the semantic phase event; this type selects W06 framing only.
#[allow(dead_code)] // W09 is the first caller.
pub(crate) enum MarkerEvent {
    Enter,
    Complete,
}

/// Initialize, emit the identity-bearing start line, then publish availability.
#[allow(dead_code)] // W09 calls this at Console.enter.
pub(crate) fn bring_up_early_console() -> Result<(), ConsoleError> {
    if CHANNEL_AVAILABLE.load(Ordering::Relaxed) {
        panic!("early console initialized twice");
    }
    console_init()?;
    let line = channel_start_line();
    console_write_bytes(line.as_bytes());
    console_write_bytes(b"\r\n");
    // Single boot CPU, masked interrupts, one publication. No cross-CPU
    // happens-before relationship is needed at P1.
    CHANNEL_AVAILABLE.store(true, Ordering::Relaxed);
    Ok(())
}

fn console_init() -> Result<(), ConsoleError> {
    let control = read_register(CONTROL);
    if control & REQUIRED_CONTROL != REQUIRED_CONTROL {
        write_register(CONTROL, control | REQUIRED_CONTROL);
    }
    if read_register(CONTROL) & REQUIRED_CONTROL != REQUIRED_CONTROL {
        return Err(ConsoleError::EnableReadbackMismatch);
    }
    Ok(())
}

fn channel_start_line() -> BoundedLine {
    let mut line = BoundedLine::new();
    let _ = write!(
        line,
        "ZELYR P1 CHANNEL ready ident=profile={} rev={} dirty={} {} {}",
        BUILD_IDENTITY.build_profile,
        BUILD_IDENTITY.source_revision,
        BUILD_IDENTITY.dirty,
        BUILD_IDENTITY.project_version,
        BUILD_IDENTITY.target_architecture,
    );
    line
}

#[allow(dead_code)] // W09 replay and W07 fatal route consume this signal.
pub(crate) fn channel_available() -> bool {
    CHANNEL_AVAILABLE.load(Ordering::Relaxed)
}

/// Return a complete bounded marker; W09 decides when to transport it.
#[allow(dead_code)] // W09 owns the transition call site.
pub(crate) fn format_marker(phase_label: &str, event: MarkerEvent) -> BoundedLine {
    let event = match event {
        MarkerEvent::Enter => "enter",
        MarkerEvent::Complete => "complete",
    };
    let mut line = BoundedLine::new();
    let _ = write!(line, "ZELYR P1 PHASE {phase_label} {event}");
    line
}

/// Transport a producer-owned line and one CRLF. Overlong content has an
/// explicit suffix, so a partial line cannot look complete to the runner.
#[allow(dead_code)] // W03/W07 producers are linked by later packages.
pub(crate) fn transport_line(line: &str) {
    if line.len() <= LINE_CAPACITY {
        console_write_bytes(line.as_bytes());
    } else {
        let mut count = LINE_CAPACITY - TRUNCATED.len();
        while !line.is_char_boundary(count) {
            count -= 1;
        }
        console_write_bytes(&line.as_bytes()[..count]);
        console_write_bytes(TRUNCATED);
    }
    console_write_bytes(b"\r\n");
}

#[allow(dead_code)] // W07 directly transports its bounded report lines.
pub(crate) fn console_write_line(line: &str) {
    transport_line(line);
}

fn console_write_bytes(bytes: &[u8]) {
    for &byte in bytes {
        while read_register(FLAGS) & TX_FULL != 0 {
            core::hint::spin_loop();
        }
        write_register(DATA, u32::from(byte));
    }
}

fn read_register(offset: u64) -> u32 {
    debug_assert!(matches!(offset, FLAGS | CONTROL));
    // SAFETY: the W01 canonical platform has a 4 KiB PL011 at the fixed
    // base; these are aligned readable 32-bit registers. W08 maps the window
    // Device-nGnRE after the MMU transition. A false platform premise is
    // FC-PLATFORM; no external value selects an address. Inventory U-010.
    unsafe { ptr::read_volatile((CONSOLE_REGION_BASE + offset) as *const u32) }
}

fn write_register(offset: u64, value: u32) {
    debug_assert!(matches!(offset, DATA | CONTROL));
    // SAFETY: the same fixed PL011 window has aligned writable 32-bit DR
    // and CR registers; one masked boot CPU writes, and W08 retains Device
    // attributes. A false platform premise is FC-PLATFORM. Inventory U-010.
    unsafe { ptr::write_volatile((CONSOLE_REGION_BASE + offset) as *mut u32, value) }
}

pub(crate) struct BoundedLine {
    bytes: [u8; LINE_CAPACITY],
    len: usize,
}

impl BoundedLine {
    fn new() -> Self {
        Self {
            bytes: [0; LINE_CAPACITY],
            len: 0,
        }
    }
    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
    #[allow(dead_code)] // W09 consumes marker text after it is linked.
    pub(crate) fn as_str(&self) -> &str {
        core::str::from_utf8(self.as_bytes()).expect("bounded line copies valid UTF-8")
    }
}

impl Write for BoundedLine {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let mut count = (LINE_CAPACITY - self.len).min(text.len());
        while !text.is_char_boundary(count) {
            count -= 1;
        }
        self.bytes[self.len..self.len + count].copy_from_slice(&text.as_bytes()[..count]);
        self.len += count;
        Ok(())
    }
}
