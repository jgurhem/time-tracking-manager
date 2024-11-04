use std::collections::HashMap;

use chrono::{DateTime, TimeDelta, Utc};

use rand::prelude::Distribution;
use rand::rngs::StdRng;
use rand::{distributions::Uniform, SeedableRng};

use super::{MyTable, Table, Tabler};

pub struct Proportional {}

///
/// Compute table containing the daily sum for each entry related to same project and task divided by the sum of entries
///
impl<'a> Tabler<'a> for Proportional {
    type Table = MyTable<u8>
    where
        Self: 'a;

    fn process(entries: Vec<crate::entries::Entry>) -> Self::Table {
        let mut delta: MyTable<TimeDelta> = MyTable::default();
        let mut days: HashMap<DateTime<Utc>, TimeDelta> = HashMap::new();

        for e in entries {
            let d = e.get_start_day();

            if let Some(old) = delta.insert(e.to_project_task(), d, e.duration()) {
                let new = delta.get_mut(e.to_project_task(), d).unwrap();
                *new += old;
            }

            if let Some(old) = days.insert(d, e.duration()) {
                let new = days.get_mut(&d).unwrap();
                *new += old;
            }
        }

        let mut table = Self::Table::default();

        for s in delta.row_headers() {
            for d in delta.col_headers() {
                let x = delta.get(s.to_string(), *d);
                if x.is_zero() {
                    continue;
                }
                let v = 100 * x.num_seconds() / days.get(d).unwrap().num_seconds();
                let v = u8::try_from(v).unwrap();
                table.insert(s.to_string(), *d, v);
            }
        }

        // Compute sum per day
        let mut days: HashMap<DateTime<Utc>, u8> = HashMap::new();

        for s in table.row_headers() {
            for d in table.col_headers() {
                let v = table.get(s.to_string(), *d);

                if let Some(old) = days.insert(*d, v) {
                    let new = days.get_mut(d).unwrap();
                    *new += old;
                }
            }
        }

        let mut rng = StdRng::seed_from_u64(1);

        // Randomly adjust values so that total per day is 100
        for d in days.clone().keys() {
            let n = *days.entry(*d).or_insert(100);
            if n == 100 {
                continue;
            }

            let mut rows = Vec::new();
            for row in table.row_headers.clone().into_iter() {
                if table.get(row.to_string(), *d) != 0 {
                    rows.push(row.clone());
                }
            }

            let dis = Uniform::new(0, rows.len());

            for _ in 0..100 - n {
                *table
                    .get_mut(rows[dis.sample(&mut rng)].to_string(), *d)
                    .unwrap() += 1;
            }
        }

        table
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use crate::entries::Entry;

    use super::*;

    #[test]
    fn entry_default() {
        let day = Utc.with_ymd_and_hms(2024, 10, 12, 0, 0, 0).unwrap();
        let entries = vec![Entry {
            start: day.checked_add_signed(TimeDelta::hours(13)).unwrap(),
            end: day.checked_add_signed(TimeDelta::hours(14)).unwrap(),
            ..Default::default()
        }];
        let table = Proportional::process(entries);
        assert_eq!(table.col_headers().len(), 1);
        assert_eq!(table.row_headers().len(), 1);
    }

    #[test]
    fn one_day_half_time() {
        let p1 = String::from("project1");
        let p2 = String::from("project2");
        let t1 = String::from("task1");
        let day = Utc.with_ymd_and_hms(2024, 10, 12, 0, 0, 0).unwrap();

        let e2 = Entry {
            project: p2.clone(),
            start: day.checked_add_signed(TimeDelta::hours(13)).unwrap(),
            end: day.checked_add_signed(TimeDelta::hours(15)).unwrap(),
            ..Default::default()
        };

        let entries = vec![
            Entry {
                project: p1.clone(),
                task: t1.clone(),
                start: day.checked_add_signed(TimeDelta::hours(11)).unwrap(),
                end: day.checked_add_signed(TimeDelta::hours(12)).unwrap(),
                ..Default::default()
            },
            Entry {
                project: p1.clone(),
                task: t1.clone(),
                start: day.checked_add_signed(TimeDelta::hours(12)).unwrap(),
                end: day.checked_add_signed(TimeDelta::hours(13)).unwrap(),
                ..Default::default()
            },
            e2.clone(),
        ];
        let table = Proportional::process(entries);

        assert_eq!(table.col_headers().len(), 1);
        assert_eq!(table.row_headers().len(), 2);
        assert_eq!(table.get(p2.clone(), day), 50);
        assert_eq!(table.get(e2.to_project_task().to_string(), day), 50);
    }

    #[test]
    fn col_sum_should_be_100() {
        let p1 = String::from("project1");
        let p2 = String::from("project2");
        let p3 = String::from("project3");
        let day = Utc.with_ymd_and_hms(2024, 10, 12, 0, 0, 0).unwrap();

        let entries = vec![
            Entry {
                project: p1.clone(),
                start: day.checked_add_signed(TimeDelta::hours(11)).unwrap(),
                end: day.checked_add_signed(TimeDelta::hours(12)).unwrap(),
                ..Default::default()
            },
            Entry {
                project: p2.clone(),
                start: day.checked_add_signed(TimeDelta::hours(12)).unwrap(),
                end: day.checked_add_signed(TimeDelta::hours(13)).unwrap(),
                ..Default::default()
            },
            Entry {
                project: p3.clone(),
                start: day.checked_add_signed(TimeDelta::hours(13)).unwrap(),
                end: day.checked_add_signed(TimeDelta::hours(14)).unwrap(),
                ..Default::default()
            },
        ];
        let table = Proportional::process(entries);

        let sum: u8 = table.row_headers().map(|r| table.get(r.clone(), day)).sum();

        assert_eq!(table.col_headers().len(), 1);
        assert_eq!(table.row_headers().len(), 3);
        assert_eq!(sum, 100);
    }
}
