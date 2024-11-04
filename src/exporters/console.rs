use std::{
    collections::{BTreeSet, HashMap},
    error::Error,
    io::{self, Stdout, Write},
};

use chrono::{DateTime, Datelike, Utc};

use colored::{Color, Colorize};

use crate::tablers::{MyTable, Table};

use super::Exporter;

pub struct Console<W: Write> {
    writer: W,
}

impl Console<Stdout> {
    pub fn stdout_output() -> Console<Stdout> {
        Console {
            writer: io::stdout(),
        }
    }
}

fn build_month_table(
    month: &DateTime<Utc>,
    dates: &BTreeSet<DateTime<Utc>>,
    t: &MyTable<u8>,
) -> FormattedTable {
    let mut ptable = FormattedTable::default();
    let ncol = dates.len() + 1;

    let mut headers: Vec<String> = Vec::with_capacity(ncol);
    headers.push(month.format("%Y %m").to_string());
    for d in dates {
        headers.push(d.day().to_string());
    }
    let headers = headers;
    ptable.set_header(headers);

    let mut row_headers = t.row_headers().collect::<Vec<_>>();
    row_headers.sort();
    for r in row_headers {
        let mut row: Vec<String> = Vec::with_capacity(ncol);
        row.push(r.clone());

        for d in dates {
            row.push(t.get(r.clone(), *d).to_string());
        }
        ptable.add_row(row);
    }

    ptable
}

impl<'a, W: Write + 'a> Exporter<'a> for Console<W> {
    type Table = MyTable<u8>
    where
        Self: 'a;

    fn export(
        &mut self,
        table: &Self::Table,
        _: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        let months = table.group_by_month();

        for (k, v) in months.iter() {
            writeln!(self.writer, "{}", build_month_table(k, v, table))?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct FormattedTable {
    headers: Vec<String>,
    columns: usize,
    rows: Vec<Vec<String>>,
}

impl FormattedTable {
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
        write!(f, "{}", &hsep)?;

        for (i, s) in self.headers.iter().enumerate() {
            write!(f, "{0} {s:1$} ", &vsep, colsize[i])?;
        }
        write!(f, "{}\n{}", &vsep, &hsep)?;

        for row in &self.rows {
            for (i, s) in row.iter().enumerate() {
                write!(f, "{0} {s:1$} ", &vsep, colsize[i])?;
            }
            write!(f, "{}\n{}", &vsep, &hsep)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use chrono::{TimeZone, Utc};
    use colored::control::SHOULD_COLORIZE;
    use io::Cursor;

    use super::*;

    fn create_table() -> MyTable<u8> {
        let mut table = MyTable::<u8>::default();
        table.insert(
            String::from("row1"),
            Utc.with_ymd_and_hms(2024, 10, 12, 0, 0, 0).unwrap(),
            8.into(),
        );
        table.insert(
            String::from("row2"),
            Utc.with_ymd_and_hms(2024, 10, 12, 0, 0, 0).unwrap(),
            9.into(),
        );
        table.insert(
            String::from("row3"),
            Utc.with_ymd_and_hms(2024, 10, 12, 0, 0, 0).unwrap(),
            10.into(),
        );

        table.insert(
            String::from("row1"),
            Utc.with_ymd_and_hms(2024, 10, 13, 0, 0, 0).unwrap(),
            8.into(),
        );
        table.insert(
            String::from("row2"),
            Utc.with_ymd_and_hms(2024, 10, 13, 0, 0, 0).unwrap(),
            9.into(),
        );
        table.insert(
            String::from("row3"),
            Utc.with_ymd_and_hms(2024, 10, 13, 0, 0, 0).unwrap(),
            10.into(),
        );

        table.insert(
            String::from("row1"),
            Utc.with_ymd_and_hms(2024, 11, 13, 0, 0, 0).unwrap(),
            8.into(),
        );
        table.insert(
            String::from("row2"),
            Utc.with_ymd_and_hms(2024, 11, 13, 0, 0, 0).unwrap(),
            9.into(),
        );
        table.insert(
            String::from("row3"),
            Utc.with_ymd_and_hms(2024, 11, 13, 0, 0, 0).unwrap(),
            10.into(),
        );

        table
    }

    #[test]
    fn stdout_works() {
        let table = create_table();
        let display = HashMap::<String, String>::new();
        let mut csv = Console::stdout_output();

        csv.export(&table, &display).unwrap();
    }

    #[test]
    fn no_display() {
        let table = create_table();
        let display = HashMap::<String, String>::new();
        let mut v = Vec::<u8>::new();
        let writer = Cursor::new(&mut v);
        let mut csv = Console { writer };

        SHOULD_COLORIZE.set_override(true);

        csv.export(&table, &display).unwrap();

        assert_eq!(String::from_utf8_lossy(&v), String::from("\u{1b}[38;2;68;68;68m+---------+----+----+\n\u{1b}[0m\u{1b}[38;2;68;68;68m|\u{1b}[0m 2024 10 \u{1b}[38;2;68;68;68m|\u{1b}[0m 12 \u{1b}[38;2;68;68;68m|\u{1b}[0m 13 \u{1b}[38;2;68;68;68m|\u{1b}[0m\n\u{1b}[38;2;68;68;68m+---------+----+----+\n\u{1b}[0m\u{1b}[38;2;68;68;68m|\u{1b}[0m row1    \u{1b}[38;2;68;68;68m|\u{1b}[0m 8  \u{1b}[38;2;68;68;68m|\u{1b}[0m 8  \u{1b}[38;2;68;68;68m|\u{1b}[0m\n\u{1b}[38;2;68;68;68m+---------+----+----+\n\u{1b}[0m\u{1b}[38;2;68;68;68m|\u{1b}[0m row2    \u{1b}[38;2;68;68;68m|\u{1b}[0m 9  \u{1b}[38;2;68;68;68m|\u{1b}[0m 9  \u{1b}[38;2;68;68;68m|\u{1b}[0m\n\u{1b}[38;2;68;68;68m+---------+----+----+\n\u{1b}[0m\u{1b}[38;2;68;68;68m|\u{1b}[0m row3    \u{1b}[38;2;68;68;68m|\u{1b}[0m 10 \u{1b}[38;2;68;68;68m|\u{1b}[0m 10 \u{1b}[38;2;68;68;68m|\u{1b}[0m\n\u{1b}[38;2;68;68;68m+---------+----+----+\n\u{1b}[0m\n\u{1b}[38;2;68;68;68m+---------+----+\n\u{1b}[0m\u{1b}[38;2;68;68;68m|\u{1b}[0m 2024 11 \u{1b}[38;2;68;68;68m|\u{1b}[0m 13 \u{1b}[38;2;68;68;68m|\u{1b}[0m\n\u{1b}[38;2;68;68;68m+---------+----+\n\u{1b}[0m\u{1b}[38;2;68;68;68m|\u{1b}[0m row1    \u{1b}[38;2;68;68;68m|\u{1b}[0m 8  \u{1b}[38;2;68;68;68m|\u{1b}[0m\n\u{1b}[38;2;68;68;68m+---------+----+\n\u{1b}[0m\u{1b}[38;2;68;68;68m|\u{1b}[0m row2    \u{1b}[38;2;68;68;68m|\u{1b}[0m 9  \u{1b}[38;2;68;68;68m|\u{1b}[0m\n\u{1b}[38;2;68;68;68m+---------+----+\n\u{1b}[0m\u{1b}[38;2;68;68;68m|\u{1b}[0m row3    \u{1b}[38;2;68;68;68m|\u{1b}[0m 10 \u{1b}[38;2;68;68;68m|\u{1b}[0m\n\u{1b}[38;2;68;68;68m+---------+----+\n\u{1b}[0m\n"));
    }
}
