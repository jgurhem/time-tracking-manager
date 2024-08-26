use chrono::{DateTime, Datelike, Days, Months, TimeZone, Utc};
use clap::Parser;

fn start_month() -> DateTime<Utc> {
    let utc = Utc::now();
    Utc.with_ymd_and_hms(utc.year(), utc.month(), 1, 0, 0, 0)
        .unwrap()
}

fn end_month() -> DateTime<Utc> {
    let utc = start_month()
        .checked_add_months(Months::new(1))
        .unwrap()
        .checked_sub_days(Days::new(1))
        .unwrap();
    Utc.with_ymd_and_hms(utc.year(), utc.month(), utc.day(), 0, 0, 0)
        .unwrap()
}

fn empty_string_vec() -> Vec<String> {
    Vec::new()
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Clockify token used to retrieve entries
    #[arg(short, long)]
    pub token: String,

    /// DateTime from wich to start retrieving entries
    #[arg(short, long, default_value_t = start_month())]
    pub start: DateTime<Utc>,

    /// DateTime until entries are retrieved
    #[arg(short, long, default_value_t = end_month())]
    pub end: DateTime<Utc>,

    /// Include entries with "Ignore" tag
    #[arg(short, long, default_value_t = false)]
    pub ignored: bool,

    /// Include non billable entries
    #[arg(short, long, default_value_t = false)]
    pub billable: bool,

    /// Projects and tasks to ignore during computations. 'Project' ignores all tasks from the project. 'Project___' ignores empty tasks. 'Project___Task' ignore the given task.
    #[arg(short('I'), long, default_values_t = empty_string_vec())]
    pub ignore_list: Vec<String>,

    /// 'Project1___Task1=Project2___Task2' allows to rename Project1 Task1 into Project2 Task2 before Tabler step
    #[arg(short, long, default_values_t = empty_string_vec())]
    pub rename: Vec<String>,

    /// 'Project1___Task1=Display' allows to rename Project1 Task1 into Display during export step
    #[arg(short, long, default_values_t = empty_string_vec())]
    pub display: Vec<String>,
}
