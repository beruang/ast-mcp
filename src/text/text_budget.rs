//! Text budget tracker.
//!
//! Used by tools that assemble composite responses (e.g. `ast_context_pack`,
//! `ast_context_for_range`) and must honour a global byte budget.

/// Tracks how many bytes have been consumed and whether a budget is exceeded.
#[derive(Debug, Clone)]
pub struct TextBudget {
    remaining: usize,
    pub exceeded: bool,
}

impl TextBudget {
    pub fn new(limit: usize) -> Self {
        Self { remaining: limit, exceeded: false }
    }

    /// Try to consume `bytes` from the budget.  Returns `true` if the charge
    /// was accepted, `false` if the budget was already exceeded or the charge
    /// would exceed it.
    pub fn try_spend(&mut self, bytes: usize) -> bool {
        if self.exceeded {
            return false;
        }
        if bytes > self.remaining {
            self.exceeded = true;
            self.remaining = 0;
            return false;
        }
        self.remaining -= bytes;
        true
    }

    /// Remaining byte allowance.
    pub fn remaining(&self) -> usize {
        self.remaining
    }
}

/// Truncate `text` to at most `max_bytes`, ensuring we don't cut in the
/// middle of a multi-byte character.
pub fn truncate_text(text: &str, max_bytes: usize) -> (&str, bool) {
    if text.len() <= max_bytes {
        return (text, false);
    }
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    (&text[..end], true)
}
