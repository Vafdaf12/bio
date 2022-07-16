use crossterm::event::{poll, read, Event};
use std::io;
use std::time::Duration;

/// Helper struct to provide iterator-based event
/// handling when working with events in [crossterm]
pub struct EventIter {
    timeout: Duration,
}

impl EventIter {
    pub fn new(timeout: Duration) -> Self {
        EventIter { timeout }
    }

    pub fn from_millis(timeout: u64) -> Self {
        EventIter {
            timeout: Duration::from_millis(timeout),
        }
    }
}

impl Default for EventIter {
    fn default() -> Self {
        EventIter::from_millis(50)
    }
}

impl Iterator for EventIter {
    type Item = io::Result<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        match poll(self.timeout) {
            Err(e) => Some(Err(e)),
            Ok(m) => m.then(|| read()),
        }
    }
}
