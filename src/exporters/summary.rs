use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    error::Error,
    fs::{create_dir_all, File},
    io::Write,
};

use chrono::Datelike;
use itertools::Itertools;
use serde::Serialize;

use crate::{
    entries::Entry,
    tablers::{MyTable, Table},
};

use super::{
    aggregated::{parse_level, AggregationLevel},
    Exporter,
};

const TOP_DESCRIPTIONS: usize = 5;
const MAX_DESCRIPTION_LEN: usize = 50;

/// Exports one CSV row per aggregated key with metrics describing the work done
pub struct Summary {
    level: AggregationLevel,
}

impl Summary {
    pub fn new(options: &[String]) -> Summary {
        Summary {
            level: parse_level(options),
        }
    }
}

#[derive(Serialize, Debug, PartialEq)]
struct Row {
    project: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    hours: String,
    share_pct: String,
    cumulative_pct: String,
    days: usize,
    months_active: usize,
    first_day: String,
    last_day: String,
    entries: usize,
    billable_hours: String,
    tags: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_descriptions: Option<String>,
    #[serde(skip)]
    minutes: i64,
}

fn group_key(entry: &Entry, level: &AggregationLevel) -> (String, String, String) {
    match level {
        AggregationLevel::Project => (entry.project.clone(), String::new(), String::new()),
        AggregationLevel::ProjectTask => (entry.project.clone(), entry.task.clone(), String::new()),
        AggregationLevel::ProjectTaskDescription => (
            entry.project.clone(),
            entry.task.clone(),
            entry.description.clone(),
        ),
    }
}

fn hours(minutes: i64) -> String {
    format!("{:.1}", minutes as f64 / 60.0)
}

fn pct(part: i64, total: i64) -> String {
    if total == 0 {
        return String::from("0.0");
    }
    format!("{:.1}", part as f64 * 100.0 / total as f64)
}

/// Removes URLs and truncates long descriptions so that they stay readable in a summary
fn shorten(description: &str) -> String {
    let short = description
        .split_whitespace()
        .filter(|w| !w.starts_with("http://") && !w.starts_with("https://"))
        .join(" ");
    let short = short.trim_end_matches([' ', '-', '>', ':']);
    if short.is_empty() {
        return String::from("(no description)");
    }
    if short.chars().count() > MAX_DESCRIPTION_LEN {
        let truncated: String = short.chars().take(MAX_DESCRIPTION_LEN).collect();
        format!("{}…", truncated.trim_end())
    } else {
        short.to_string()
    }
}

fn summarize(entries: &[Entry], level: &AggregationLevel) -> Vec<Row> {
    let total_minutes: i64 = entries.iter().map(|e| e.duration().num_minutes()).sum();
    let groups = entries.iter().into_group_map_by(|e| group_key(e, level));

    let mut rows: Vec<Row> = groups
        .into_iter()
        .map(|((project, task, description), entries)| {
            let minutes: i64 = entries.iter().map(|e| e.duration().num_minutes()).sum();
            let billable_minutes: i64 = entries
                .iter()
                .filter(|e| e.billable)
                .map(|e| e.duration().num_minutes())
                .sum();
            let days: BTreeSet<_> = entries.iter().map(|e| e.get_start_day()).collect();
            let months: BTreeSet<_> = days.iter().map(|d| (d.year(), d.month())).collect();
            let tags: BTreeSet<_> = entries.iter().flat_map(|e| e.tags.iter()).collect();

            let top_descriptions = if *level == AggregationLevel::ProjectTaskDescription {
                None
            } else {
                let mut by_description: BTreeMap<String, i64> = BTreeMap::new();
                for e in &entries {
                    *by_description.entry(shorten(&e.description)).or_insert(0) +=
                        e.duration().num_minutes();
                }
                Some(
                    by_description
                        .into_iter()
                        .sorted_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)))
                        .take(TOP_DESCRIPTIONS)
                        .map(|(d, m)| format!("{} {}h ({}%)", d, hours(m), pct(m, minutes)))
                        .join(" | "),
                )
            };

            Row {
                project,
                task: (*level != AggregationLevel::Project).then_some(task),
                description: (*level == AggregationLevel::ProjectTaskDescription)
                    .then_some(description),
                hours: hours(minutes),
                share_pct: pct(minutes, total_minutes),
                cumulative_pct: String::new(),
                days: days.len(),
                months_active: months.len(),
                first_day: days
                    .first()
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default(),
                last_day: days
                    .last()
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default(),
                entries: entries.len(),
                billable_hours: hours(billable_minutes),
                tags: tags.into_iter().join(";"),
                top_descriptions,
                minutes,
            }
        })
        .collect();

    rows.sort_by(|a, b| {
        b.minutes.cmp(&a.minutes).then_with(|| {
            (&a.project, &a.task, &a.description).cmp(&(&b.project, &b.task, &b.description))
        })
    });

    let mut cumulative = 0;
    for row in &mut rows {
        cumulative += row.minutes;
        row.cumulative_pct = pct(cumulative, total_minutes);
    }
    rows
}

fn write_rows<W: Write>(rows: &[Row], writer: W) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_writer(writer);
    for row in rows {
        wtr.serialize(row)?;
    }
    wtr.flush()?;
    Ok(())
}

impl<'a> Exporter<'a> for Summary {
    type Table
        = MyTable<u8>
    where
        Self: 'a;

    fn export(
        &mut self,
        table: &Self::Table,
        _: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        let entries = table.entries();
        let (Some(first), Some(last)) = (
            entries.iter().map(|e| e.get_start_day()).min(),
            entries.iter().map(|e| e.get_start_day()).max(),
        ) else {
            return Ok(());
        };

        create_dir_all("export")?;
        let path = format!(
            "export/summary_{}_{}.csv",
            first.format("%Y-%m-%d"),
            last.format("%Y-%m-%d")
        );
        write_rows(&summarize(entries, &self.level), File::create(path)?)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn make_entry(
        project: &str,
        task: &str,
        description: &str,
        (month, day): (u32, u32),
        hours: u32,
        billable: bool,
        tags: &[&str],
    ) -> Entry {
        Entry {
            project: project.to_string(),
            task: task.to_string(),
            description: description.to_string(),
            billable,
            tags: tags.iter().map(|t| t.to_string()).collect(),
            start: Utc.with_ymd_and_hms(2026, month, day, 8, 0, 0).unwrap(),
            end: Utc
                .with_ymd_and_hms(2026, month, day, 8 + hours, 0, 0)
                .unwrap(),
            ..Default::default()
        }
    }

    fn entries() -> Vec<Entry> {
        vec![
            make_entry("proj-a", "task1", "design", (1, 5), 2, true, &["x"]),
            make_entry("proj-a", "task1", "design", (1, 5), 1, true, &[]),
            make_entry("proj-a", "task1", "impl", (3, 10), 4, false, &["y", "x"]),
            make_entry("proj-a", "task2", "review", (2, 1), 1, true, &[]),
            make_entry("proj-b", "", "support", (2, 2), 2, true, &[]),
        ]
    }

    #[test]
    fn summarize_project_task() {
        let rows = summarize(&entries(), &AggregationLevel::ProjectTask);
        assert_eq!(rows.len(), 3);

        let first = &rows[0];
        assert_eq!(first.project, "proj-a");
        assert_eq!(first.task.as_deref(), Some("task1"));
        assert_eq!(first.description, None);
        assert_eq!(first.hours, "7.0");
        assert_eq!(first.share_pct, "70.0");
        assert_eq!(first.cumulative_pct, "70.0");
        assert_eq!(first.entries, 3);
        assert_eq!(first.days, 2);
        assert_eq!(first.first_day, "2026-01-05");
        assert_eq!(first.last_day, "2026-03-10");
        assert_eq!(first.months_active, 2);
        assert_eq!(first.billable_hours, "3.0");
        assert_eq!(first.tags, "x;y");
        assert_eq!(
            first.top_descriptions.as_deref(),
            Some("impl 4.0h (57.1%) | design 3.0h (42.9%)")
        );

        assert_eq!(rows[1].project, "proj-b");
        assert_eq!(rows[1].cumulative_pct, "90.0");
        assert_eq!(rows[2].task.as_deref(), Some("task2"));
        assert_eq!(rows[2].cumulative_pct, "100.0");
    }

    #[test]
    fn summarize_project() {
        let rows = summarize(&entries(), &AggregationLevel::Project);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].project, "proj-a");
        assert_eq!(rows[0].task, None);
        assert_eq!(rows[0].hours, "8.0");
        assert_eq!(
            rows[0].top_descriptions.as_deref(),
            Some("impl 4.0h (50.0%) | design 3.0h (37.5%) | review 1.0h (12.5%)")
        );
    }

    #[test]
    fn summarize_description_has_no_top_descriptions() {
        let rows = summarize(&entries(), &AggregationLevel::ProjectTaskDescription);
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0].description.as_deref(), Some("impl"));
        assert!(rows.iter().all(|r| r.top_descriptions.is_none()));
    }

    #[test]
    fn summarize_empty() {
        assert!(summarize(&[], &AggregationLevel::Project).is_empty());
    }

    #[test]
    fn shorten_description() {
        assert_eq!(shorten(""), "(no description)");
        assert_eq!(shorten("plan aws - https://www.notion.so/page"), "plan aws");
        assert_eq!(shorten("tilt -> https://tilt.dev/"), "tilt");
        assert_eq!(shorten("https://example.org"), "(no description)");
        assert_eq!(shorten(&"a".repeat(60)), format!("{}…", "a".repeat(50)));
    }

    #[test]
    fn top_descriptions_merge_shortened() {
        let entries = vec![
            make_entry("p", "t", "plan aws - https://a", (1, 5), 1, true, &[]),
            make_entry("p", "t", "plan aws - https://b", (1, 6), 2, true, &[]),
        ];
        let rows = summarize(&entries, &AggregationLevel::ProjectTask);
        assert_eq!(
            rows[0].top_descriptions.as_deref(),
            Some("plan aws 3.0h (100.0%)")
        );
    }

    #[test]
    fn write_csv_project_task() {
        let rows = summarize(
            &[make_entry("p", "t", "d", (1, 5), 2, true, &[])],
            &AggregationLevel::ProjectTask,
        );
        let mut v = Vec::<u8>::new();
        write_rows(&rows, &mut v).unwrap();
        assert_eq!(
            String::from_utf8(v).unwrap(),
            "project,task,hours,share_pct,cumulative_pct,days,months_active,first_day,last_day,entries,billable_hours,tags,top_descriptions\n\
             p,t,2.0,100.0,100.0,1,1,2026-01-05,2026-01-05,1,2.0,,d 2.0h (100.0%)\n"
        );
    }

    #[test]
    fn write_csv_project_task_description() {
        let rows = summarize(
            &[make_entry("p", "t", "d", (1, 5), 2, true, &[])],
            &AggregationLevel::ProjectTaskDescription,
        );
        let mut v = Vec::<u8>::new();
        write_rows(&rows, &mut v).unwrap();
        assert_eq!(
            String::from_utf8(v).unwrap(),
            "project,task,description,hours,share_pct,cumulative_pct,days,months_active,first_day,last_day,entries,billable_hours,tags\n\
             p,t,d,2.0,100.0,100.0,1,1,2026-01-05,2026-01-05,1,2.0,\n"
        );
    }
}
