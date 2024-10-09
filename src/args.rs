use chrono::{DateTime, Datelike, TimeZone, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::utils::end_of_month;

fn start_month() -> DateTime<Utc> {
    let utc = Utc::now();
    Utc.with_ymd_and_hms(utc.year(), utc.month(), 1, 0, 0, 0)
        .unwrap()
}

fn end_month() -> DateTime<Utc> {
    end_of_month(&start_month())
}

#[derive(Parser, Debug, Serialize, Deserialize, PartialEq)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Clockify token used to retrieve entries
    #[arg(short, long)]
    pub token: String,

    /// DateTime from wich to start retrieving entries
    #[arg(short, long, default_value_t = start_month())]
    #[serde(default = "start_month")]
    pub start: DateTime<Utc>,

    /// DateTime until entries are retrieved
    #[arg(short, long, default_value_t = end_month())]
    #[serde(default = "end_month")]
    pub end: DateTime<Utc>,

    /// Include entries with "Ignore" tag
    #[arg(short, long, default_value_t = false)]
    #[serde(default)]
    pub ignored: bool,

    /// Include non billable entries
    #[arg(short, long, default_value_t = false)]
    #[serde(default)]
    pub billable: bool,

    /// Projects and tasks to ignore during computations. 'Project' ignores all tasks from the project. 'Project___' ignores empty tasks. 'Project___Task' ignore the given task.
    #[arg(short('I'), long, default_values_t = Vec::<String>::new())]
    #[serde(default)]
    pub ignore_list: Vec<String>,

    /// 'Project1___Task1=Project2___Task2' allows to rename Project1 Task1 into Project2 Task2 before Tabler step
    #[arg(short, long, default_values_t = Vec::<String>::new())]
    #[serde(default)]
    pub rename: Vec<String>,

    /// 'Project1___Task1=Display' allows to rename Project1 Task1 into Display during export step
    #[arg(short, long, default_values_t = Vec::<String>::new())]
    #[serde(default)]
    pub display: Vec<String>,
}

impl Args {
    pub fn from_token(token: String) -> Args {
        Args {
            token,
            ..Default::default()
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            token: Default::default(),
            start: start_month(),
            end: end_month(),
            ignored: false,
            billable: false,
            ignore_list: Default::default(),
            rename: Default::default(),
            display: Default::default(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_deserialization() {
        let args = Args::from_token("token".into());
        assert_eq!(args, serde_json::from_str("{\"token\":\"token\"}".into()).expect("valid json representing Args"))
    }

}
