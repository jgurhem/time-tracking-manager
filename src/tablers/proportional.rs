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

            match delta.insert(e.to_project___task(), d, e.duration()) {
                Some(old) => {
                    let new = delta.get_mut(e.to_project___task(), d).unwrap();
                    *new += old;
                }
                None => {}
            }

            match days.insert(d, e.duration()) {
                Some(old) => {
                    let new = days.get_mut(&d).unwrap();
                    *new += old;
                }
                None => {}
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

                match days.insert(*d, v) {
                    Some(old) => {
                        let new = days.get_mut(&d).unwrap();
                        *new += old;
                    }
                    None => {}
                }
            }
        }

        let mut rng = StdRng::seed_from_u64(1);

        // Randomly adjust values so that total per day is 100
        for d in days.clone().keys().into_iter() {
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
