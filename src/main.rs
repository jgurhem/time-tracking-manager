use clap::Parser;
use std::{collections::HashMap, error::Error};
use time_tracking_manager::{
    args::Args,
    entries::Entry,
    exporters::{console::Console, csv::CSV, Exporter},
    filters::{predicate_filter, FilterParam},
    providers::{clockify::Clockify, Provider},
    renamers::Renames,
    tablers::{proportional::Proportional, Tabler}, utils,
};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    dbg!(&args);

    let mut c = Clockify::new(args.token.as_str());
    let entries = c.load(args.start, args.end)?;

    let param = FilterParam::build(&args);
    let renames = Renames::build(&args).unwrap();
    let entries: Vec<Entry> = entries
        .into_iter()
        .filter(|x| predicate_filter(&x, &param))
        .map(|x| renames.predicate_rename(x))
        .collect();

    for e in &entries {
        println!("entry {:?}", e);
    }

    let result = Proportional::process(entries);
    let mut display = HashMap::new();

    for d in args.display.iter() {
        let (k, v)=utils::split_eq(d).unwrap();
        display.insert(k.to_string(), v.to_string());
    }

    Console::export(&result, &display);
    CSV::export(&result, &display);
    Ok(())
}
