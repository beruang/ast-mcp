use std::sync::atomic::{AtomicBool, Ordering};

pub struct Cancellable {
    cancelled: AtomicBool,
}

impl Default for Cancellable {
    fn default() -> Self {
        Self::new()
    }
}

impl Cancellable {
    pub fn new() -> Self {
        Cancellable { cancelled: AtomicBool::new(false) }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}
