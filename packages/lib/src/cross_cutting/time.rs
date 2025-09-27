use chrono::Utc;
use serde::Serialize;

#[derive(Serialize)]
pub struct DateTime(chrono::DateTime<Utc>);

impl DateTime {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn timestamp(&self) -> String {
        self.0.to_rfc3339()
    }
}
