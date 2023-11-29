pub mod parser;
pub use parser::*;
use time::{Date, Duration, UtcOffset};

pub const DATE_DISPLAY_FORMATTING: &str = "%Y-%m-%d %H:%M:%S";

pub fn today(offset: UtcOffset) -> Date {
    time::OffsetDateTime::now_utc().to_offset(offset).date()
}

pub fn tomorrow(offset: UtcOffset) -> Date {
    day_from_today(offset, 1)
}

pub fn day_from_today(offset: UtcOffset, n: i64) -> Date {
    today(offset).saturating_add(Duration::days(n))
}
