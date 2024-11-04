use std::fmt::Debug;

use chrono::{DateTime, Datelike, TimeDelta, TimeZone, Utc};

#[derive(Debug, PartialEq, Default, Clone)]
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

    pub fn to_project_task(&self) -> String {
        if self.task.is_empty() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_default() {
        let entry = Entry::default();
        assert_eq!(entry.duration(), TimeDelta::zero())
    }

    #[test]
    fn duration_1day() {
        let entry = Entry {
            start: Utc.with_ymd_and_hms(2024, 10, 12, 9, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2024, 10, 13, 9, 0, 0).unwrap(),
            ..Default::default()
        };
        assert_eq!(entry.duration(), TimeDelta::days(1))
    }

    #[test]
    fn duration_negative() {
        let entry = Entry {
            start: Utc.with_ymd_and_hms(2024, 10, 13, 9, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2024, 10, 12, 9, 0, 0).unwrap(),
            ..Default::default()
        };
        assert_eq!(entry.duration(), TimeDelta::days(-1))
    }

    #[test]
    fn project_only() {
        let project = String::from("project");
        let entry = Entry {
            project: project.clone(),
            ..Default::default()
        };
        assert_eq!(entry.to_project_task(), project)
    }

    #[test]
    fn project_task() {
        let project = String::from("project");
        let task = String::from("task");
        let entry = Entry {
            project: project.clone(),
            task: task.clone(),
            ..Default::default()
        };
        let pt = entry.to_project_task();

        assert!(pt.contains(&project), "{} should contains {}", pt, project);
        assert!(pt.contains(&task), "{} should contains {}", pt, task);
    }

    #[test]
    fn get_start_day() {
        let now = Utc.with_ymd_and_hms(2024, 10, 12, 10, 11, 12).unwrap();
        let entry = Entry {
            start: now,
            ..Default::default()
        };
        let start = Utc.with_ymd_and_hms(2024, 10, 12, 0, 0, 0).unwrap();
        assert_eq!(entry.get_start_day(), start)
    }
}
