use clap::Parser;
use std::error::Error;
use time_tracking_manager::{
    args::Args,
    entries::Entry,
    exporters::{console::Console, Exporter},
    filters::{predicate_filter, FilterParam},
    providers::{clockify::Clockify, Provider},
    renamers::Renames,
    tablers::{proportional::Proportional, Tabler},
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
    Console::export(&result);
    CSV::export(&result);
    Ok(())
}
