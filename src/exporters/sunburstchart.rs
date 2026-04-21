use crate::entries::{Entry, Work};
use crate::exporters::Exporter;
use crate::tablers::{MyTable, Table};
use std::collections::HashMap;
use std::error::Error;

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
    entries.sort_by(|a, b| a.to_extended_desc().cmp(&b.to_extended_desc()));

    let entries = HashMap::<String, Entry>::from_iter(
        table
            .entries()
            .iter()
            .into_group_map_by(|e| e.to_extended_desc())
            .iter()
            .map(|(k, v)| {
                let mut e = v.first().unwrap().clone().clone();

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
        entries
            .values()
            .into_group_map_by(|e| e.to_project_task())
            .into_iter(),
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
                    value += table.get(work.clone(), col.clone()) as i32;
                }

                if value > 0 {
                    let text = if work.task.is_empty() {
                        work.project.clone()
                    } else {
                        work.task.clone()
                    };
                    
                    let mut node =
                        SunburstNode::new(text).value(value as f64);

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
                                let d = e.duration().num_minutes() as f64 / total_duration * value as f64;
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

impl<'a> Exporter<'a> for SunburstChart {
    type Table = MyTable<u8>;

    fn export(
        &mut self,
        table: &Self::Table,
        _: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        // Implementation for exporting to Sunburst format goes here

        let chart = Chart::new()
            .title(Title::new().text("Sunburst Chart").left("center"))
            .series(
                Sunburst::new()
                    .data(convert(&table))
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

        // Chart dimension 1000x800.
        let mut renderer = HtmlRenderer::new("my charts", 1000, 800);
        // Save the chart as HTML file.
        renderer.save(&chart, "export/chart.html")?;

        Ok(())
    }
}
