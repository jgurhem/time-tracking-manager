use clap::Parser;
use std::error::Error;
use time_tracking_manager::{
    args::Args,
    filters::{predicate_filter, FilterParam},
    providers::{clockify::Clockify, Provider},
    renamers::Renames,
};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    dbg!(&args);

    let mut c = Clockify::new(args.token.as_str());
    let entries = c.load(args.start, args.end)?;

    let param = FilterParam::build(&args);
    let renames = Renames::build(&args).unwrap();
    let entries = entries
        .into_iter()
        .filter(|x| predicate_filter(&x, &param))
        .map(|x| renames.predicate_rename(x));

    for e in entries {
        println!("entry {:?}", e);
    }

    Ok(())
}
