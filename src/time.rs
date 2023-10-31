use chrono::{DateTime, Local};

pub const DATE_DISPLAY_FORMATTING: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeRange {
    pub from: Option<DateTime<Local>>,
    pub to: Option<DateTime<Local>>,
}

impl TimeRange {
    pub fn intersects_with(&self, point: chrono::DateTime<Local>) -> bool {
        if self.from.is_some_and(|fr| fr > point) {
            return false;
        }
        if self.to.is_some_and(|to| to < point) {
            return false;
        }
        true
    }
}
