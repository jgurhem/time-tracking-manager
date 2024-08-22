use std::collections::HashMap;

use chrono::{DateTime, Datelike, TimeZone, Utc};
use comfy_table::Table as PTable;

use crate::tablers::{MyTable, Table};

use super::Exporter;

pub struct Console {}

fn group_by_month(
    headers: std::collections::hash_set::Iter<DateTime<Utc>>,
) -> HashMap<DateTime<Utc>, Vec<DateTime<Utc>>> {
    let mut groups: HashMap<DateTime<Utc>, Vec<DateTime<Utc>>> = HashMap::new();

    for h in headers {
        let m = Utc
            .with_ymd_and_hms(h.year(), h.month(), 1, 0, 0, 0)
            .unwrap();

        if groups.contains_key(&m) {
            groups
                .get_mut(&m)
                .expect("Item should be present because we checked it before")
                .push(h.clone());
        } else {
            groups.insert(m, vec![h.clone()]);
        }
    }

    groups
}

fn build_month_table(month: &DateTime<Utc>, dates: &Vec<DateTime<Utc>>, t: &MyTable<u8>) -> PTable {
    let mut ptable = PTable::new();
    let mut dates = dates.clone();
    let ncol = dates.len() + 1;
    dates.sort();

    let mut headers: Vec<String> = Vec::with_capacity(ncol);
    headers.push(month.to_string());
    for d in &dates {
        headers.push(d.day().to_string());
    }
    let headers = headers;
    ptable.set_header(headers);

    for r in t.row_headers() {
        let mut row: Vec<String> = Vec::with_capacity(ncol);
        row.push(r.clone());

        for d in &dates {
            row.push(t.get(r.clone(), *d).to_string());
        }
        ptable.add_row(row);
    }

    ptable
}

impl<'a> Exporter<'a> for Console {
    type Table = MyTable<u8>
    where
        Self: 'a;

    fn export(table: &Self::Table) {
        let months = group_by_month(table.col_headers());

        for (k, v) in months.iter() {
            println!("{}", build_month_table(&k, &v, table));
        }
    }
}
