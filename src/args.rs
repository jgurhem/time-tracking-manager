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

#[derive(Parser, Debug, Serialize, Deserialize, PartialEq, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Provider used to retrieve entries
    #[arg(short('P'), long)]
    pub provider: String,

    /// Options passed to the provider such as authentication informations and token
    /// ---
    /// *Clockify Options*
    ///   token: Clockify authentication token
    #[arg(short, long)]
    #[serde(default)]
    pub provider_options: Vec<String>,

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

    /// Number of slots per day
    #[arg(short, long, default_value_t = default_granularity())]
    #[serde(default = "default_granularity")]
    pub granularity: u8,

    /// Number of slots per day
    #[arg(long, default_value = "")]
    #[serde(default)]
    pub period: String,
}

const fn default_granularity() -> u8 {
    100
}

impl Default for Args {
    fn default() -> Self {
        Self {
            provider: "clockify".into(),
            provider_options: Default::default(),
            start: start_month(),
            end: end_month(),
            ignored: false,
            billable: false,
            ignore_list: Default::default(),
            rename: Default::default(),
            display: Default::default(),
            granularity: default_granularity(),
            period: String::from(""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_deserialization() {
        let args = Args::default();
        assert_eq!(
            args,
            serde_json::from_str("{\"provider\":\"clockify\"}")
                .expect("valid json representing Args")
        )
    }
}
