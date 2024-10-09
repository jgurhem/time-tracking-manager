use clap::Parser;
use std::{borrow::BorrowMut, collections::HashMap, error::Error};
use time_tracking_manager::{
    args::Args,
    entries::Entry,
    exporters::{console::Console, csv::CSV, Exporter},
    filters::{predicate_filter, FilterParam},
    providers::{Provider, ProviderHandle},
    renamers::Renames,
    tablers::{proportional::Proportional, Tabler},
    utils,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    dbg!(&args);

    let handle = ProviderHandle::new("clockify", args).expect("Provider should be available");
    let args = handle.args;
    let mut provider = handle.provider.borrow_mut();
    let entries = provider.load(args.start, args.end).await?;

    let param = FilterParam::build(&args);
    let renames = Renames::build(&args).unwrap();
    let entries: Vec<Entry> = entries
        .into_iter()
        .filter(|x| predicate_filter(&x, &param))
        .map(|x| renames.predicate_rename(x))
        .collect();

    let result = Proportional::process(entries);
    let mut display = HashMap::new();

    for d in args.display.iter() {
        let (k, v) = utils::split_eq(d).unwrap();
        display.insert(k.to_string(), v.to_string());
    }

    Console {}.export(&result, &display).unwrap();
    CSV {}.export(&result, &display).unwrap();
    Ok(())
}
