use std::collections::HashMap;

use chrono::{DateTime, TimeDelta, Utc};

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
        let mut delta: MyTable<TimeDelta> = MyTable::new();
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

        let mut table = Self::Table::new();

        for s in delta.row_headers() {
            for d in delta.col_headers() {
                let x = delta.get(s.to_string(), *d);
                if x.is_zero() {
                    continue;
                }
                let v = 100 * x.num_seconds() / days.get(d).unwrap().num_seconds();
                println!("{s} {d} {v}");
                table.insert(s.to_string(), *d, v.try_into().unwrap());
            }
        }

        // todo : ensure the values in each columns are equal to 100

        table
    }
}
