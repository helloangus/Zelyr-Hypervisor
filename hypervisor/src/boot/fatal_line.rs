//! Pure fixed-capacity fatal-report line formatter, shared with host tests.
use core::fmt::{self, Write};

const CAPACITY: usize = 128;
const TRUNCATED: &str = " [truncated]";

pub(crate) struct Line {
    bytes: [u8; CAPACITY],
    len: usize,
    truncated: bool,
}
impl Line {
    pub(crate) fn new() -> Self {
        Self {
            bytes: [0; CAPACITY],
            len: 0,
            truncated: false,
        }
    }
    pub(crate) fn as_str(&self) -> &str {
        // Every write copies a UTF-8 prefix and an ASCII suffix. Keep the
        // fatal path nonpanicking if a future writer breaks that invariant.
        match core::str::from_utf8(&self.bytes[..self.len]) {
            Ok(line) => line,
            Err(_) => "ZELYR P1 invalid-line",
        }
    }
}
impl Write for Line {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if self.truncated {
            return Ok(());
        }
        // Reserve the suffix before every fragment: a later fragment can
        // never silently cut the line after it looked complete.
        let room = CAPACITY - TRUNCATED.len() - self.len;
        let mut count = room.min(text.len());
        while !text.is_char_boundary(count) {
            count -= 1;
        }
        self.bytes[self.len..self.len + count].copy_from_slice(&text.as_bytes()[..count]);
        self.len += count;
        if count != text.len() {
            let suffix = TRUNCATED.as_bytes();
            self.bytes[self.len..self.len + suffix.len()].copy_from_slice(suffix);
            self.len += suffix.len();
            self.truncated = true;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_short_line_is_preserved() {
        let mut line = Line::new();
        let _ = write!(line, "fatal {}", 7);
        assert_eq!(line.as_str(), "fatal 7");
    }

    #[test]
    fn multibyte_truncation_is_explicit_and_stable() {
        let mut line = Line::new();
        let _ = line.write_str(&"中".repeat(100));
        let result = line.as_str();
        assert!(result.ends_with(TRUNCATED));
        assert!(result.len() <= CAPACITY);
        assert_eq!((result.len() - TRUNCATED.len()) % 3, 0);
        let before = result.to_owned();
        let _ = line.write_str("later");
        assert_eq!(line.as_str(), before);
    }

    #[test]
    fn later_fragment_cannot_silently_overflow() {
        let mut line = Line::new();
        let _ = line.write_str(&"a".repeat(CAPACITY - TRUNCATED.len()));
        let _ = line.write_str("b");
        assert_eq!(line.as_str().len(), CAPACITY);
        assert!(line.as_str().ends_with(TRUNCATED));
    }
}
