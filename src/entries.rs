use std::fmt::Debug;

use chrono::{DateTime, Datelike, TimeDelta, TimeZone, Utc};

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
    pub fn duration(&self) -> TimeDelta {
        self.end - self.start
    }

    pub fn to_project___task(&self) -> String {
        if self.task == "" {
            self.project.to_string().clone()
        } else {
            format!("{}___{}", &self.project, &self.task)
        }
    }

    pub fn get_start_day(&self) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(
            self.start.year(),
            self.start.month(),
            self.start.day(),
            0,
            0,
            0,
        )
        .unwrap()
    }
}
