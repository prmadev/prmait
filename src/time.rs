use chrono::{DateTime, Datelike, Days, Local, TimeZone};

pub const DATE_DISPLAY_FORMATTING: &str = "%Y-%m-%d %H:%M:%S";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeRange {
    pub from: Option<DateTime<Local>>,
    pub to: Option<DateTime<Local>>,
}

impl TimeRange {
    pub fn build(
        from: Option<DateTime<Local>>,
        to: Option<DateTime<Local>>,
    ) -> Result<Self, Error> {
        if let Some(fr) = from {
            if let Some(t) = to {
                if !fr.lt(&t) {
                    return Err(Error::ToIsNotAfterFrom);
                }
            }
        }
        Ok(Self { from, to })
    }

    pub fn intersects_with(&self, point: chrono::DateTime<Local>) -> bool {
        if self.from.is_some_and(|fr| fr > point) {
            return false;
        }
        if self.to.is_some_and(|to| to < point) {
            return false;
        }
        true
    }
    pub fn is_after(&self, point: chrono::DateTime<Local>) -> bool {
        if self.from.is_some_and(|fr| fr > point) {
            return true;
        }
        false
    }
    pub fn day_range_of_time(point: &chrono::DateTime<Local>) -> Result<Self, Error> {
        let fr = chrono::Local
            .with_ymd_and_hms(point.year(), point.month(), point.day(), 0, 0, 0)
            .single()
            .ok_or(Error::CreatingTimeFailed)?;

        let to = fr
            .checked_add_days(Days::new(1))
            .ok_or(Error::AddingTimeFailed)?;

        Ok(Self {
            from: Some(fr),
            to: Some(to),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not get time")]
    CreatingTimeFailed,
    #[error("failed in adding time to day")]
    AddingTimeFailed,
    #[error("from must be before after")]
    ToIsNotAfterFrom,
}

#[cfg(test)]
mod testing {
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<Error>();
    }
}
