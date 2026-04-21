use crate::entries::{Entry, Work};
use crate::exporters::Exporter;
use crate::tablers::{MyTable, Table};
use std::collections::HashMap;
use std::error::Error;

use charming::datatype::{DataPoint, DataPointItem};
use charming::series::Pie;
use charming::HtmlRenderer;
use charming::{
    component::Title,
    element::{Emphasis, EmphasisFocus, ItemStyle, Label, LabelPosition, Sort},
    series::{Sunburst, SunburstLevel, SunburstNode},
    Chart,
};
use chrono::TimeDelta;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SunburstChart {}

fn convert(table: &MyTable<u8>) -> Vec<SunburstNode> {
    let mut entries = table.entries().clone();
    entries.sort_by_key(|a| a.to_extended_desc());

    let entries = HashMap::<String, Entry>::from_iter(
        table
            .entries()
            .iter()
            .into_group_map_by(|e| e.to_extended_desc())
            .iter()
            .map(|(k, v)| {
                let mut e = (*v.first().unwrap()).clone();

                for entry in v.iter().skip(1) {
                    let d = entry.duration().num_minutes() as u32;
                    let prev = e.duration().num_minutes() as u32;
                    let new_end = e.start + TimeDelta::minutes((d + prev) as i64);
                    e.end = new_end;
                }

                (k.clone(), e)
            }),
    );

    let entries = HashMap::<Work, Vec<&Entry>>::from_iter(
        entries.values().into_group_map_by(|e| e.to_project_task()),
    );

    table
        .row_headers()
        .sorted_by(|a, b| a.project.cmp(&b.project))
        .chunk_by(|r| r.project.clone())
        .into_iter()
        .map(|(project, works)| {
            let mut children = Vec::new();
            for work in works {
                let mut value: i32 = 0;
                for col in table.col_headers() {
                    value += table.get(work.clone(), *col) as i32;
                }

                if value > 0 {
                    let text = if work.task.is_empty() {
                        work.project.clone()
                    } else {
                        work.task.clone()
                    };

                    let mut node = SunburstNode::new(text).value(value as f64);

                    let total_duration = entries
                        .get(work)
                        .unwrap_or(&Vec::new())
                        .iter()
                        .map(|e| e.duration().num_minutes() as f64)
                        .sum::<f64>();

                    node = node.children(
                        entries
                            .get(work)
                            .unwrap_or(&Vec::new())
                            .iter()
                            .map(|e| {
                                let d = e.duration().num_minutes() as f64 / total_duration
                                    * value as f64;
                                SunburstNode::new(e.description.clone()).value(d)
                            })
                            .collect(),
                    );

                    children.push(node);
                }
            }
            if !children.is_empty() {
                SunburstNode::new(project).children(children)
            } else {
                SunburstNode::new(project)
            }
        })
        .collect()
}

fn to_dataframe(table: &MyTable<u8>) -> Vec<DataPoint> {
    let total = table
        .entries()
        .iter()
        .map(|e| e.duration().num_minutes())
        .sum::<i64>();
    table
        .entries()
        .iter()
        .into_group_map_by(|e| e.to_project_task())
        .iter()
        .map(|(k, v)| {
            let s = v
                .iter()
                .map(|e| e.duration())
                .sum::<TimeDelta>()
                .num_minutes();
            (
                s,
                format!("{} ({:.2}%)", k, (s as f64 / total as f64) * 100.0),
            )
        })
        .sorted_by(|a, b| b.0.cmp(&a.0))
        .map(|(k, v)| DataPoint::Item(DataPointItem::new(k).name(v)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entries::Entry;
    use crate::tablers::proportional::Proportional;
    use crate::tablers::Tabler;
    use chrono::{TimeDelta, TimeZone, Utc};

    fn make_entry(
        project: &str,
        task: &str,
        description: &str,
        start_hour: u32,
        hours: i64,
    ) -> Entry {
        let start = Utc
            .with_ymd_and_hms(2024, 10, 12, start_hour, 0, 0)
            .unwrap();
        Entry {
            project: project.to_string(),
            task: task.to_string(),
            description: description.to_string(),
            start,
            end: start + TimeDelta::hours(hours),
            ..Default::default()
        }
    }

    #[test]
    fn convert_empty() {
        let table = Proportional::default().process(vec![]);
        assert!(convert(&table).is_empty());
    }

    #[test]
    fn convert_project_only() {
        let entry = make_entry("proj", "", "meeting", 9, 2);
        let table = Proportional::default().process(vec![entry]);
        let nodes = convert(&table);
        // project-only: text = project name; one description child
        let expected = SunburstNode::new("proj").children(vec![SunburstNode::new("proj")
            .value(100.0)
            .children(vec![SunburstNode::new("meeting").value(100.0)])]);
        assert_eq!(nodes, vec![expected]);
    }

    #[test]
    fn convert_project_with_task() {
        let entry = make_entry("proj", "task", "work", 9, 2);
        let table = Proportional::default().process(vec![entry]);
        let nodes = convert(&table);
        // task present: text = task name
        let expected = SunburstNode::new("proj").children(vec![SunburstNode::new("task")
            .value(100.0)
            .children(vec![SunburstNode::new("work").value(100.0)])]);
        assert_eq!(nodes, vec![expected]);
    }

    #[test]
    fn convert_two_projects() {
        // Equal durations → each gets 50 slots; sorted by project name
        let e1 = make_entry("proj1", "", "meeting", 9, 4);
        let e2 = make_entry("proj2", "", "review", 13, 4);
        let table = Proportional::default().process(vec![e1, e2]);
        let nodes = convert(&table);
        let expected = vec![
            SunburstNode::new("proj1").children(vec![SunburstNode::new("proj1")
                .value(50.0)
                .children(vec![SunburstNode::new("meeting").value(50.0)])]),
            SunburstNode::new("proj2").children(vec![SunburstNode::new("proj2")
                .value(50.0)
                .children(vec![SunburstNode::new("review").value(50.0)])]),
        ];
        assert_eq!(nodes, expected);
    }

    #[test]
    fn convert_merges_duplicate_descriptions() {
        // Two entries with identical extended_desc are merged; skip(1) branch is covered
        let e1 = make_entry("proj", "", "meeting", 9, 2);
        let e2 = make_entry("proj", "", "meeting", 11, 2);
        let table = Proportional::default().process(vec![e1, e2]);
        let nodes = convert(&table);
        // Still one project node, one description child with value 100
        let expected = SunburstNode::new("proj").children(vec![SunburstNode::new("proj")
            .value(100.0)
            .children(vec![SunburstNode::new("meeting").value(100.0)])]);
        assert_eq!(nodes, vec![expected]);
    }

    #[test]
    fn to_dataframe_empty() {
        let table = Proportional::default().process(vec![]);
        assert!(to_dataframe(&table).is_empty());
    }

    #[test]
    fn to_dataframe_single_project() {
        let entry = make_entry("proj", "", "work", 9, 8);
        let table = Proportional::default().process(vec![entry]);
        let data = to_dataframe(&table);
        // 8h = 480 min, 100%
        assert_eq!(
            data,
            vec![DataPoint::Item(
                DataPointItem::new(480i64).name("proj (100.00%)")
            )]
        );
    }

    #[test]
    fn to_dataframe_two_projects_sorted_descending() {
        // Different durations so sort order is deterministic (larger first)
        let e1 = make_entry("proj1", "", "a", 9, 6); // 360 min, 75%
        let e2 = make_entry("proj2", "", "b", 15, 2); // 120 min, 25%
        let table = Proportional::default().process(vec![e1, e2]);
        let data = to_dataframe(&table);
        assert_eq!(data.len(), 2);
        assert_eq!(
            data[0],
            DataPoint::Item(DataPointItem::new(360i64).name("proj1 (75.00%)"))
        );
        assert_eq!(
            data[1],
            DataPoint::Item(DataPointItem::new(120i64).name("proj2 (25.00%)"))
        );
    }
}

impl<'a> Exporter<'a> for SunburstChart {
    type Table = MyTable<u8>;

    fn export(
        &mut self,
        table: &Self::Table,
        _: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        // Implementation for exporting to Sunburst format goes here

        let sb = Chart::new()
            .title(Title::new().text("Sunburst Chart").left("center"))
            .series(
                Sunburst::new()
                    .data(convert(table))
                    .name("Work Distribution")
                    .levels(vec![
                        SunburstLevel::new()
                            .item_style(ItemStyle::new().border_width(4))
                            .label(Label::new().position(LabelPosition::Inside))
                            .r0("15%")
                            .r("30%"),
                        SunburstLevel::new()
                            .item_style(ItemStyle::new().border_width(4))
                            .label(Label::new().position(LabelPosition::Inside))
                            .r0("30%")
                            .r("50%"),
                        SunburstLevel::new()
                            .item_style(ItemStyle::new().border_width(4))
                            .label(Label::new().position(LabelPosition::Inside))
                            .r0("50%")
                            .r("70%"),
                        SunburstLevel::new()
                            .item_style(ItemStyle::new().border_width(4))
                            .label(Label::new().position(LabelPosition::Inside))
                            .r0("70%")
                            .r("100%"),
                    ])
                    .sort(Sort::Descending)
                    .emphasis(
                        Emphasis::new()
                            .focus(EmphasisFocus::Ancestor)
                            .item_style(ItemStyle::new().border_width(3)),
                    ),
            );

        let pie = Chart::new()
            .title(Title::new().text("Pie Chart").left("center"))
            .series(
                Pie::new()
                    .name("Access From")
                    .radius("70%")
                    .data(to_dataframe(table))
                    .emphasis(
                        Emphasis::new()
                            .item_style(
                                ItemStyle::new()
                                    .shadow_blur(10)
                                    .shadow_offset_x(0)
                                    .shadow_color("rgba(0, 0, 0, 0.5)"),
                            )
                            .label(Label::new().show(true).font_size(16).font_weight("bold")),
                    ),
            );

        // Chart dimension 1000x800.
        let mut renderer = HtmlRenderer::new("my charts", 1200, 800);
        // Save the chart as HTML file.
        renderer.save(&sb, "export/chart.html")?;
        renderer.save(&pie, "export/pie.html")?;

        Ok(())
    }
}
