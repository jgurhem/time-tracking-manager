use std::fmt::{Debug, Display};

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
    pub absolute: Option<f64>,
}

#[derive(Debug, PartialEq, Default, Clone, Eq, Hash, Ord, PartialOrd)]
pub struct Work {
    pub task: String,
    pub project: String,
}

impl Display for Work {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.task.is_empty() {
            write!(f, "{}", self.project)
        } else {
            write!(f, "{}___{}", self.project, self.task)
        }
    }
}

impl Work {
    pub fn new(combined: String) -> Self {
        let parts: Vec<&str> = combined.split("___").collect();
        Work {
            project: parts.first().unwrap_or(&"").to_string(),
            task: parts.get(1).unwrap_or(&"").to_string(),
        }
    }
}

impl Entry {
    pub fn duration(&self) -> TimeDelta {
        self.end - self.start
    }

    pub fn to_project_task(&self) -> Work {
        Work {
            project: self.project.clone(),
            task: self.task.clone(),
        }
    }

    pub fn to_extended_desc(&self) -> String {
        format!("{}{}", self.to_project_task(), self.description)
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
        assert_eq!(
            entry.to_project_task(),
            Work {
                project,
                task: String::new()
            }
        )
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

        assert_eq!(pt.project, project);
        assert_eq!(pt.task, task);
    }

    #[test]
    fn work_display_project_only() {
        let w = Work {
            project: String::from("proj"),
            task: String::new(),
        };
        assert_eq!(w.to_string(), "proj");
    }

    #[test]
    fn work_display_project_task() {
        let w = Work {
            project: String::from("proj"),
            task: String::from("task"),
        };
        assert_eq!(w.to_string(), "proj___task");
    }

    #[test]
    fn work_new_project_only() {
        let w = Work::new(String::from("proj"));
        assert_eq!(w.project, "proj");
        assert_eq!(w.task, "");
    }

    #[test]
    fn work_new_project_task() {
        let w = Work::new(String::from("proj___task"));
        assert_eq!(w.project, "proj");
        assert_eq!(w.task, "task");
    }

    #[test]
    fn extended_desc() {
        let entry = Entry {
            project: String::from("proj"),
            task: String::from("task"),
            description: String::from("desc"),
            ..Default::default()
        };
        assert_eq!(entry.to_extended_desc(), "proj___taskdesc");
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
