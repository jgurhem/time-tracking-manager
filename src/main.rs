use chrono::{DateTime, Datelike, Days, Months, TimeZone, Utc};
use clap::Parser;
use futures::executor::block_on;
use time_tracking_manager::providers::{clockify::Clockify, Provider};

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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Clockify token used to retrieve entries
    #[arg(short, long)]
    token: String,

    /// DateTime from wich to start retrieving entries
    #[arg(short, long, default_value_t = start_month())]
    start: DateTime<Utc>,

    /// DateTime until entries are retrieved
    #[arg(short, long, default_value_t = end_month())]
    end: DateTime<Utc>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    dbg!(&args);

    let mut c = Clockify::new(args.token);
    let entries = c.load(args.start, args.end);

    let entries = block_on(entries).unwrap();

    for e in entries{
        println!("entry {:?}", e);
    }
}
