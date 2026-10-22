use chrono::{DateTime, FixedOffset, Utc};

pub fn now() -> DateTime<FixedOffset> {
    Utc::now().fixed_offset()
}
