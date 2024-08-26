use std::collections::HashMap;

use chrono::Datelike;
use csv::Writer;

use crate::tablers::{MyTable, Table};

use super::Exporter;

pub struct CSV {}

impl<'a> Exporter<'a> for CSV {
    type Table = MyTable<u8>
    where
        Self: 'a;

    fn export(table: &Self::Table, display: &HashMap<String, String>) {
        let months = table.group_by_month();

        for (month, dates) in months.iter() {
            let ncol = dates.len() + 1;
            let mut wtr =
                Writer::from_path(format!("export/{}_{}.csv", month.year(), month.month()))
                    .unwrap();

            let mut headers: Vec<String> = Vec::with_capacity(ncol);
            headers.push(month.format("%Y %m").to_string());
            for d in dates {
                headers.push(d.day().to_string());
            }
            let headers = headers;
            wtr.write_record(headers).unwrap();

            let mut row_headers = table.row_headers().collect::<Vec<_>>();
            row_headers.sort();
            for r in row_headers {
                let mut row: Vec<String> = Vec::with_capacity(ncol);
                row.push(display.get(r).unwrap_or(r).clone());

                for d in dates {
                    row.push(table.get(r.clone(), *d).to_string());
                }
                wtr.write_record(row).unwrap();
            }
            wtr.flush().unwrap();
        }
    }
}
