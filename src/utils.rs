use chrono::{DateTime, Months, TimeDelta, Utc};

use crate::errors::SplitError;

pub fn split___(s: &str) -> (String, String) {
    let s: Vec<&str> = s.split("___").collect();
    if s.len() == 1 {
        (s[0].to_string(), String::new())
    } else {
        (s[0].to_string(), s[1].to_string())
    }
}

pub fn split_eq(s: &str) -> Result<(String, String), SplitError> {
    let split: Vec<&str> = s.split("=").collect();
    match split.len() {
        2 => Ok((split[0].to_string(), split[1].to_string())),
        _ => Err(SplitError {
            field: s.to_string(),
            reason: String::from("field should contain one and only one (key=value)"),
        }),
    }
}

pub fn end_of_month(date: &DateTime<Utc>) -> DateTime<Utc> {
    date.checked_add_months(Months::new(1))
        .unwrap()
        .checked_sub_signed(TimeDelta::milliseconds(1))
        .unwrap()
}
