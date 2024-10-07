use std::error::Error;

use chrono::{DateTime, Months, TimeDelta, Utc};

pub fn split___(s: &str) -> (&str, &str) {
    let s: Vec<&str> = s.split("___").collect();
    if s.len() == 1 {
        (s[0], "")
    } else {
        (s[0], s[1])
    }
}

pub fn split_eq(s: &str) -> Result<(&str, &str), Box<dyn Error>> {
    let s: Vec<&str> = s.split("=").collect();
    match s.len() {
        2 => Ok((s[0], s[1])),
        1 => Err(Box::from("Rename should have an = in the middle")),
        _ => Err(Box::from("Rename should have only one = in the middle")),
    }
}

pub fn end_of_month(date: &DateTime<Utc>) -> DateTime<Utc> {
    date.checked_add_months(Months::new(1))
        .unwrap()
        .checked_sub_signed(TimeDelta::milliseconds(1))
        .unwrap()
}
