use std::{
    collections::{BTreeMap, HashMap},
    io::{self, Stdout, Write},
};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::tablers::{MyTable, Table};

use super::{console::FormattedTable, Exporter};

#[derive(ValueEnum, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum AggregationLevel {
    Project,
    ProjectTask,
    #[default]
    ProjectTaskDescription,
}

impl std::fmt::Display for AggregationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.to_possible_value()
                .expect("no skipped variants")
                .get_name()
        )
    }
}

pub struct Aggregated<W: Write> {
    writer: W,
    level: AggregationLevel,
}

fn parse_level(options: &[String]) -> AggregationLevel {
    options
        .iter()
        .find_map(|o| {
            let (k, v) = o.split_once('=')?;
            if k == "aggregation_level" {
                AggregationLevel::from_str(v, true).ok()
            } else {
                None
            }
        })
        .unwrap_or_default()
}

impl Aggregated<Stdout> {
    pub fn stdout_output(options: &[String]) -> Aggregated<Stdout> {
        Aggregated {
            writer: io::stdout(),
            level: parse_level(options),
        }
    }
}

impl<'a, W: Write + 'a> Exporter<'a> for Aggregated<W> {
    type Table
        = MyTable<u8>
    where
        Self: 'a;

    fn export(
        &mut self,
        table: &Self::Table,
        _: &HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entries = table.entries();
        let mut map: BTreeMap<String, i64> = BTreeMap::new();

        for entry in entries {
            let key = match self.level {
                AggregationLevel::Project => entry.project.clone(),
                AggregationLevel::ProjectTask => {
                    if entry.task.is_empty() {
                        entry.project.clone()
                    } else {
                        format!("{}/{}", entry.project, entry.task)
                    }
                }
                AggregationLevel::ProjectTaskDescription => {
                    format!("{}/{}/{}", entry.project, entry.task, entry.description)
                }
            };
            *map.entry(key).or_insert(0) += entry.duration().num_minutes();
        }

        let mut ptable = FormattedTable::default();
        ptable.set_header(vec!["Key".to_string(), "Hours".to_string()]);

        let mut total_minutes: i64 = 0;
        for (key, minutes) in &map {
            let hours = *minutes as f64 / 60.0;
            ptable.add_row(vec![key.clone(), format!("{:.1}h", hours)]);
            total_minutes += minutes;
        }

        let total_hours = total_minutes as f64 / 60.0;
        ptable.add_row(vec!["Total".to_string(), format!("{:.1}h", total_hours)]);

        writeln!(self.writer, "{}", ptable)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use std::collections::HashMap;
    use std::io::Cursor;

    use crate::entries::Entry;
    use crate::tablers::MyTable;

    use super::*;

    fn make_entry(project: &str, task: &str, description: &str, hours: i64) -> Entry {
        Entry {
            project: project.to_string(),
            task: task.to_string(),
            description: description.to_string(),
            start: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            end: Utc
                .with_ymd_and_hms(2024, 1, 1, hours as u32, 0, 0)
                .unwrap(),
            ..Default::default()
        }
    }

    #[test]
    fn aggregate_by_project() {
        let entries = vec![
            make_entry("proj-a", "task1", "desc", 2),
            make_entry("proj-a", "task2", "desc", 3),
            make_entry("proj-b", "", "desc", 1),
        ];
        let table = MyTable::<u8>::new(entries);
        let display = HashMap::new();
        let mut v = Vec::<u8>::new();
        let writer = Cursor::new(&mut v);
        let mut exporter = Aggregated {
            writer,
            level: AggregationLevel::Project,
        };
        exporter.export(&table, &display).unwrap();
        let output = String::from_utf8_lossy(&v);
        assert!(output.contains("proj-a"));
        assert!(output.contains("proj-b"));
        assert!(output.contains("5.0h"));
        assert!(output.contains("1.0h"));
        assert!(output.contains("6.0h")); // total
    }

    #[test]
    fn aggregate_by_project_task_no_task() {
        let entries = vec![make_entry("proj-a", "", "desc", 2)];
        let table = MyTable::<u8>::new(entries);
        let display = HashMap::new();
        let mut v = Vec::<u8>::new();
        let writer = Cursor::new(&mut v);
        let mut exporter = Aggregated {
            writer,
            level: AggregationLevel::ProjectTask,
        };
        exporter.export(&table, &display).unwrap();
        let output = String::from_utf8_lossy(&v);
        assert!(output.contains("proj-a"));
        assert!(!output.contains("proj-a/"));
    }

    #[test]
    fn aggregate_by_project_task_with_task() {
        let entries = vec![
            make_entry("proj-a", "task1", "desc", 2),
            make_entry("proj-a", "task1", "other", 3),
            make_entry("proj-a", "task2", "desc", 3),
        ];
        let table = MyTable::<u8>::new(entries);
        let display = HashMap::new();
        let mut v = Vec::<u8>::new();
        let writer = Cursor::new(&mut v);
        let mut exporter = Aggregated {
            writer,
            level: AggregationLevel::ProjectTask,
        };
        exporter.export(&table, &display).unwrap();
        let output = String::from_utf8_lossy(&v);
        assert!(output.contains("proj-a/task1"));
        assert!(output.contains("proj-a/task2"));
        assert!(output.contains("5.0h"));
        assert!(output.contains("8.0h")); // total
    }

    #[test]
    fn aggregate_by_project_task_description() {
        let entries = vec![
            make_entry("proj-a", "task1", "desc1", 2),
            make_entry("proj-a", "task1", "desc1", 1),
            make_entry("proj-a", "task1", "desc2", 3),
        ];
        let table = MyTable::<u8>::new(entries);
        let display = HashMap::new();
        let mut v = Vec::<u8>::new();
        let writer = Cursor::new(&mut v);
        let mut exporter = Aggregated {
            writer,
            level: AggregationLevel::ProjectTaskDescription,
        };
        exporter.export(&table, &display).unwrap();
        let output = String::from_utf8_lossy(&v);
        assert!(output.contains("proj-a/task1/desc1"));
        assert!(output.contains("proj-a/task1/desc2"));
        assert!(output.contains("3.0h"));
        assert!(output.contains("6.0h")); // total
    }

    #[test]
    fn parse_level_valid_option() {
        let options = vec!["aggregation_level=project".to_string()];
        assert_eq!(parse_level(&options), AggregationLevel::Project);

        let options = vec!["aggregation_level=project-task".to_string()];
        assert_eq!(parse_level(&options), AggregationLevel::ProjectTask);

        let options = vec!["aggregation_level=project-task-description".to_string()];
        assert_eq!(
            parse_level(&options),
            AggregationLevel::ProjectTaskDescription
        );
    }

    #[test]
    fn parse_level_no_option() {
        assert_eq!(parse_level(&[]), AggregationLevel::ProjectTaskDescription);
    }

    #[test]
    fn parse_level_unrelated_option() {
        let options = vec!["other_key=project".to_string()];
        assert_eq!(
            parse_level(&options),
            AggregationLevel::ProjectTaskDescription
        );
    }

    #[test]
    fn aggregation_level_display() {
        assert_eq!(AggregationLevel::Project.to_string(), "project");
        assert_eq!(AggregationLevel::ProjectTask.to_string(), "project-task");
        assert_eq!(
            AggregationLevel::ProjectTaskDescription.to_string(),
            "project-task-description"
        );
    }
}
