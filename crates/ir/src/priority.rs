//! Priority comparison and ordering

use crate::types::Priority;

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight().cmp(&other.weight())
    }
}

impl Priority {
    /// Determine if this priority is "critical" (P1 or >= 90)
    pub fn is_critical(&self) -> bool {
        matches!(self, Priority::P1) || matches!(self, Priority::Custom(w) if *w >= 90)
    }

    /// Determine if this priority is "high" (P2 or >= 70)
    pub fn is_high(&self) -> bool {
        matches!(self, Priority::P1 | Priority::P2) || matches!(self, Priority::Custom(w) if *w >= 70)
    }
}
