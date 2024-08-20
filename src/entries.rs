use chrono::{DateTime, TimeDelta, Utc};

#[derive(Debug, PartialEq)]
pub struct Entry {
    pub id: String,
    pub description: String,
    pub billable: bool,
    pub project: String,
    pub task: String,
    pub tags: Vec<String>,
    pub end: DateTime<Utc>,
    pub start: DateTime<Utc>,
}

impl Entry {
    fn duration(self) -> TimeDelta {
        self.end - self.start
    }
}
