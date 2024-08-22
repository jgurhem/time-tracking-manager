use std::collections::HashMap;

use chrono::{DateTime, Datelike, TimeZone, Utc};

use colored::{Color, Colorize};

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

        groups
            .entry(m)
            .and_modify(|e| e.push(h.clone()))
            .or_insert(vec![h.clone()]);
    }

    groups
}

fn build_month_table(
    month: &DateTime<Utc>,
    dates: &Vec<DateTime<Utc>>,
    t: &MyTable<u8>,
) -> FormattedTable {
    let mut ptable = FormattedTable::new();
    let mut dates = dates.clone();
    let ncol = dates.len() + 1;
    dates.sort();

    let mut headers: Vec<String> = Vec::with_capacity(ncol);
    headers.push(month.format("%Y %m").to_string());
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

pub struct FormattedTable {
    headers: Vec<String>,
    columns: usize,
    rows: Vec<Vec<String>>,
}

impl FormattedTable {
    pub fn new() -> FormattedTable {
        FormattedTable {
            headers: Vec::new(),
            columns: 0,
            rows: Vec::new(),
        }
    }

    pub fn set_header(&mut self, headers: Vec<String>) {
        if self.columns < headers.len() {
            self.columns = headers.len();
        }
        self.headers = headers;
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        if self.columns < row.len() {
            self.columns = row.len();
        }
        self.rows.push(row);
    }
}

impl std::fmt::Display for FormattedTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = Color::TrueColor {
            r: 68,
            g: 68,
            b: 68,
        };
        let mut colsize: Vec<usize> = vec![0; self.columns];
        for (i, h) in self.headers.iter().enumerate() {
            colsize[i] = h.len();
        }

        for row in &self.rows {
            for (i, s) in row.iter().enumerate() {
                let l = s.len();
                if colsize[i] < l {
                    colsize[i] = l;
                }
            }
        }

        let hsepsise: usize = colsize.iter().sum();
        let mut hsep = String::with_capacity(2 + colsize.len() * 3 + hsepsise);
        hsep.push('+');
        for s in &colsize {
            hsep.push('-');
            for _ in 0..*s {
                hsep.push('-');
            }
            hsep.push('-');
            hsep.push('+');
        }
        hsep.push('\n');
        let hsep = hsep.color(color);
        let vsep = "|".color(color);
        f.write_fmt(format_args!("{}", &hsep))?;

        for (i, s) in self.headers.iter().enumerate() {
            write!(f, "{0} {s:1$} ", &vsep, colsize[i])?;
        }
        f.write_fmt(format_args!("{}\n{}", &vsep, &hsep))?;

        for row in &self.rows {
            for (i, s) in row.iter().enumerate() {
                write!(f, "{0} {s:1$} ", &vsep, colsize[i])?;
            }
            f.write_fmt(format_args!("{}\n{}", &vsep, &hsep))?;
        }

        Ok(())
    }
}
