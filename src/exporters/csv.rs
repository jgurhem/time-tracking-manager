use std::{collections::HashMap, error::Error, fs::create_dir_all};

use chrono::Datelike;
use csv::Writer;

use crate::tablers::{MyTable, Table};

use super::Exporter;

pub struct CSV {}

impl<'a> Exporter<'a> for CSV {
    type Table = MyTable<u8>
    where
        Self: 'a;

    fn export(
        &mut self,
        table: &Self::Table,
        display: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        let months = table.group_by_month();

        for (month, dates) in months.iter() {
            let ncol = dates.len() + 1;
            create_dir_all("export").ok();
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{read_to_string, remove_file};

    use chrono::{TimeZone, Utc};
    use serial_test::serial;

    use super::*;

    fn create_table() -> MyTable<u8> {
        let mut table = MyTable::<u8>::default();
        table.insert(
            String::from("row1"),
            Utc.with_ymd_and_hms(2024, 10, 12, 0, 0, 0).unwrap(),
            8,
        );
        table.insert(
            String::from("row2"),
            Utc.with_ymd_and_hms(2024, 10, 12, 0, 0, 0).unwrap(),
            9,
        );
        table.insert(
            String::from("row3"),
            Utc.with_ymd_and_hms(2024, 10, 12, 0, 0, 0).unwrap(),
            10,
        );

        table.insert(
            String::from("row1"),
            Utc.with_ymd_and_hms(2024, 10, 13, 0, 0, 0).unwrap(),
            8,
        );
        table.insert(
            String::from("row2"),
            Utc.with_ymd_and_hms(2024, 10, 13, 0, 0, 0).unwrap(),
            9,
        );
        table.insert(
            String::from("row3"),
            Utc.with_ymd_and_hms(2024, 10, 13, 0, 0, 0).unwrap(),
            10,
        );

        table.insert(
            String::from("row1"),
            Utc.with_ymd_and_hms(2024, 11, 13, 0, 0, 0).unwrap(),
            8,
        );
        table.insert(
            String::from("row2"),
            Utc.with_ymd_and_hms(2024, 11, 13, 0, 0, 0).unwrap(),
            9,
        );
        table.insert(
            String::from("row3"),
            Utc.with_ymd_and_hms(2024, 11, 13, 0, 0, 0).unwrap(),
            10,
        );

        table
    }

    #[test]
    #[serial]
    fn csv_no_display() {
        let table = create_table();
        let display = HashMap::<String, String>::new();
        let mut csv = CSV {};

        let path = "export/2024_10.csv";
        remove_file(path).ok();
        csv.export(&table, &display).unwrap();
        let content = read_to_string(path).unwrap();
        remove_file(path).ok();
        assert_eq!(
            content,
            String::from("2024 10,12,13\nrow1,8,8\nrow2,9,9\nrow3,10,10\n")
        );

        let path = "export/2024_11.csv";
        remove_file(path).ok();
        csv.export(&table, &display).unwrap();
        let content = read_to_string(path).unwrap();
        remove_file(path).ok();
        assert_eq!(
            content,
            String::from("2024 11,13\nrow1,8\nrow2,9\nrow3,10\n")
        );
    }

    #[test]
    #[serial]
    fn csv_display() {
        let table = create_table();
        let mut display = HashMap::<String, String>::new();
        display.insert(String::from("row1"), String::from("displayed"));
        let mut csv = CSV {};

        let path = "export/2024_10.csv";
        remove_file(path).ok();
        csv.export(&table, &display).unwrap();
        let content = read_to_string(path).unwrap();
        remove_file(path).ok();
        assert_eq!(
            content,
            String::from("2024 10,12,13\ndisplayed,8,8\nrow2,9,9\nrow3,10,10\n")
        );

        let path = "export/2024_11.csv";
        remove_file(path).ok();
        csv.export(&table, &display).unwrap();
        let content = read_to_string(path).unwrap();
        remove_file(path).ok();
        assert_eq!(
            content,
            String::from("2024 11,13\ndisplayed,8\nrow2,9\nrow3,10\n")
        );
    }
}
