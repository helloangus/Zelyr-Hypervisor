//! Early panic route: bounded, single-entry, terminal.
//!
//! Contract: P1-W02 design, panic/identity contracts §1 and §4. The route
//! is the P1 failure sink from establishment stage 3 onward (static data
//! ready); W07's accepted design supersedes the report body through the
//! recorded extension seam — the handler registration, the guard, and the
//! bounded-stop discipline transfer only through that design.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use super::identity;
use super::writer::early_write_bytes;

/// Fixed panic-route marker prefix. Recorded in the W02 implementation
/// record before W10/W11 consume it as a marker class (W10 token precedent).
const P1_PANIC_PREFIX: &[u8] = b"ZELYR P1 PANIC\r\n";

/// Single-entry guard: on re-entry (panic inside the route) the route
/// diverges immediately into the bounded stop instead of recursing.
static PANIC_ROUTE_ENTERED: AtomicBool = AtomicBool::new(false);

/// The binary's single panic handler (`p1_panic` in the W02 design).
/// Never allocates, never re-enters, never returns.
#[panic_handler]
fn p1_panic(info: &PanicInfo<'_>) -> ! {
    // Relaxed ordering: single boot CPU, DAIF masked; the flag exists to
    // make re-entry diverge, not to synchronize memory.
    if PANIC_ROUTE_ENTERED.swap(true, Ordering::Relaxed) {
        bounded_stop();
    }

    early_write_bytes(P1_PANIC_PREFIX);
    early_write_bytes(b"identity: ");
    emit_identity_line();
    early_write_bytes(b"\r\n");

    // The current API always carries a message; the buffer bounds it to
    // the fixed capacity (truncation, never a panic).
    let mut message_buffer = BoundedBuffer::new();
    let _ = core::write!(message_buffer, "{}", info.message());
    early_write_bytes(b"message: ");
    early_write_bytes(message_buffer.as_bytes());
    early_write_bytes(b"\r\n");

    if let Some(location) = info.location() {
        let mut buffer = BoundedBuffer::new();
        let _ = core::write!(
            buffer,
            "{}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
        early_write_bytes(b"location: ");
        early_write_bytes(buffer.as_bytes());
        early_write_bytes(b"\r\n");
    }

    bounded_stop()
}

/// Terminal stop (branch-to-self): reached from the route's end and from
/// every re-entry attempt; independent of any output success.
fn bounded_stop() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

fn emit_identity_line() {
    let identity = &identity::BUILD_IDENTITY;
    let mut buffer = BoundedBuffer::new();
    // Truncation is the designed behavior; the line is static and well
    // under the capacity, the guard is for future field growth.
    let _ = core::write!(
        buffer,
        "profile={} rev={} dirty={} {} {}",
        identity.build_profile,
        identity.source_revision,
        identity.dirty,
        identity.project_version,
        identity.target_architecture,
    );
    early_write_bytes(buffer.as_bytes());
}

/// Fixed-capacity stack buffer with a minimal `core::fmt::Write` adapter
/// (W02 design §4: allocation-free formatting; overflow truncates, never
/// panics).
struct BoundedBuffer {
    bytes: [u8; Self::CAPACITY],
    len: usize,
}

impl BoundedBuffer {
    const CAPACITY: usize = 128;

    fn new() -> Self {
        BoundedBuffer {
            bytes: [0; Self::CAPACITY],
            len: 0,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl Write for BoundedBuffer {
    fn write_str(&mut self, source: &str) -> core::fmt::Result {
        let taken = source.len().min(self.bytes.len() - self.len);
        self.bytes[self.len..self.len + taken].copy_from_slice(source.as_bytes());
        self.len += taken;
        Ok(())
    }
}
