use std::collections::HashMap;

use chrono::Datelike;

use super::{MyTable, Tabler};

const SLOTS_PER_DAY: u8 = 100;

#[derive(Debug, Clone)]
pub struct Proportional {
    pub period: Period,
    pub granularity: u8,
}

#[derive(Debug, Clone, Default)]
pub enum Period {
    Day,
    Week,
    Month,
    Year,
    #[default]
    All,
}

impl Default for Proportional {
    fn default() -> Self {
        Self {
            period: Default::default(),
            granularity: SLOTS_PER_DAY,
        }
    }
}

impl<'a> Tabler<'a> for Proportional {
    type Table
        = MyTable<u8>
    where
        Self: 'a;

    fn process(&self, mut entries: Vec<crate::entries::Entry>) -> Self::Table {
        assert!(SLOTS_PER_DAY % self.granularity == 0);

        entries.sort_by_key(crate::entries::Entry::get_start_day);

        let mut table = Self::Table::default();

        let pred: fn(&crate::entries::Entry, &crate::entries::Entry) -> bool = match self.period {
            Period::Day => |a, b| a.get_start_day() == b.get_start_day(),
            Period::Week => |a, b| a.start.iso_week() == b.start.iso_week(),
            Period::Month => {
                |a, b| (a.start.year(), a.start.month()) == (b.start.year(), b.start.month())
            }
            Period::Year => |a, b| a.start.year() == b.start.year(),
            Period::All => |_, _| true,
        };

        for entries in entries.chunk_by(pred) {
            self.process_slice(&mut table, entries);
        }

        table
    }
}

impl Proportional {
    fn process_slice(&self, table: &mut MyTable<u8>, entries: &[crate::entries::Entry]) {
        let granularity_norm = SLOTS_PER_DAY / self.granularity;
        let mut total_absolute = 0.;
        let mut total_relative = 0.;
        let mut total_days = 0;
        let mut rows = HashMap::new();
        let mut days = HashMap::new();

        // Compute total number of days, and total duration worked
        for entries in entries.chunk_by(|a, b| a.get_start_day() == b.get_start_day()) {
            let day = entries.first().unwrap().get_start_day();
            let workday = entries
                .iter()
                .any(|entry| !entry.tags.contains(&String::from("out-of-office")));

            total_days += workday as i32;

            for entry in entries {
                let row = rows
                    .entry(entry.to_project_task())
                    .or_insert((0, 0., 0., vec![]));
                if let Some(duration) = entry.absolute {
                    total_absolute += duration;
                    row.1 += duration;
                } else {
                    total_relative += entry.duration().num_seconds() as f64;
                    row.2 += entry.duration().num_seconds() as f64;
                }
                row.3.push(entry);
            }

            if workday {
                days.insert(day, self.granularity as i32);
            }
        }

        let mut remaining = self.granularity as i32 * total_days;

        let norm = (total_days as f64 - total_absolute) * self.granularity as f64 / total_relative;

        for (slots, absolute, _, _) in rows.values_mut() {
            let n = (*absolute * self.granularity as f64)
                .floor()
                .min(remaining as f64);
            *slots += n as i32;
            remaining -= n as i32;
        }
        for (slots, _, relative, _) in rows.values_mut() {
            let n = (*relative * norm).floor().min(remaining as f64);
            *slots += n as i32;
            remaining -= n as i32;
        }

        let mut remainder = rows
            .values_mut()
            .map(|(slots, absolute, relative, _)| {
                let absolute = (*absolute * self.granularity as f64).fract();
                let relative = (*relative * norm).fract();
                (slots, absolute + relative)
            })
            .collect::<Vec<_>>();

        remainder
            .sort_by(|(_, f0), (_, f1)| f1.partial_cmp(f0).unwrap_or(std::cmp::Ordering::Equal));

        for (slots, fract) in remainder {
            let n = fract.ceil().min(remaining as f64);
            *slots += n as i32;
            remaining -= n as i32;
        }

        let mut rows = rows
            .iter_mut()
            .map(|(name, (slots, _, _, entries))| (name, slots, entries))
            .collect::<Vec<_>>();
        rows.sort_by_key(|(_, slots, _)| **slots);

        for (name, slots, entries) in rows.iter_mut() {
            entries.sort_by_key(|entry| entry.get_start_day());

            // Try allocate on the right days, with the right number of slots
            for entry in entries.iter_mut() {
                let Some(day) = days.get_mut(&entry.get_start_day()) else {
                    continue;
                };
                let absolute = entry.absolute.unwrap_or_default() * self.granularity as f64;
                let relative = entry.duration().num_seconds() as f64 * norm;
                let n = (absolute + relative).ceil() as i32;
                let n = n.min(**slots).min(*day);

                if let Some(old) = table.insert(
                    name.clone(),
                    entry.get_start_day(),
                    n as u8 * granularity_norm,
                ) {
                    *table.get_mut(name.clone(), entry.get_start_day()).unwrap() += old;
                }
                **slots -= n;
                *day -= n;
            }

            // Try allocate on other days where this project has been worked on
            for entry in entries.iter_mut() {
                let Some(day) = days.get_mut(&entry.get_start_day()) else {
                    continue;
                };
                let n = (**slots).min(*day);

                if let Some(old) = table.insert(
                    name.clone(),
                    entry.get_start_day(),
                    n as u8 * granularity_norm,
                ) {
                    *table.get_mut(name.clone(), entry.get_start_day()).unwrap() += old;
                }
                **slots -= n;
                *day -= n;
            }

            // Try allocate on any day
            let mut days = days.iter_mut().collect::<Vec<_>>();
            days.sort_by_key(|(_, slots)| **slots);
            for (&date, day) in days {
                let n = (**slots).min(*day);

                if let Some(old) = table.insert(name.clone(), date, n as u8 * granularity_norm) {
                    *table.get_mut(name.clone(), date).unwrap() += old;
                }
                **slots -= n;
                *day -= n;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeDelta, TimeZone, Utc};

    use crate::{entries::Entry, tablers::Table};

    use super::*;

    #[test]
    fn entry_default() {
        let day = Utc.with_ymd_and_hms(2024, 10, 12, 0, 0, 0).unwrap();
        let entries = vec![Entry {
            start: day.checked_add_signed(TimeDelta::hours(13)).unwrap(),
            end: day.checked_add_signed(TimeDelta::hours(14)).unwrap(),
            ..Default::default()
        }];
        let table = Proportional::default().process(entries);
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
        let table = Proportional::default().process(entries);

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
        let table = Proportional::default().process(entries);

        let sum: u8 = table.row_headers().map(|r| table.get(r.clone(), day)).sum();

        assert_eq!(table.col_headers().len(), 1);
        assert_eq!(table.row_headers().len(), 3);
        assert_eq!(sum, 100);
    }
}
