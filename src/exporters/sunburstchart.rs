use crate::exporters::Exporter;
use crate::tablers::{MyTable, Table};
use std::collections::HashMap;
use std::error::Error;

use charming::HtmlRenderer;
use charming::{
    component::Title,
    element::{
        Emphasis, EmphasisFocus, ItemStyle, Label, LabelPosition, Sort,
    },
    series::{Sunburst, SunburstLevel, SunburstNode},
    Chart,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SunburstChart {}

fn convert(table: &MyTable<u8>) -> Vec<SunburstNode> {
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
                    children.push(
                        SunburstNode::new(format!("{}", work.task.clone())).value(value as f64),
                    );
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
                            .item_style(ItemStyle::new().border_width(2))
                            .label(Label::new().position(LabelPosition::Inside))
                            .r0("15%")
                            .r("50%"),
                        SunburstLevel::new()
                            .item_style(ItemStyle::new().border_width(2))
                            .label(Label::new().position(LabelPosition::Inside))
                            .r0("50%")
                            .r("70%"),
                        SunburstLevel::new()
                            .item_style(ItemStyle::new().border_width(2))
                            .label(Label::new().position(LabelPosition::Inside))
                            .r0("70%")
                            .r("80%"),
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
