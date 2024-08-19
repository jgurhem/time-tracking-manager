use clap::Parser;
use futures::executor::block_on;
use time_tracking_manager::{args::Args, filters::{predicate_filter, FilterParam}, providers::{clockify::Clockify, Provider}};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    dbg!(&args);

    let mut c = Clockify::new(args.token.as_str());
    let entries = c.load(args.start, args.end);

    let entries = block_on(entries).unwrap();

    let param = FilterParam::build(&args);
    let entries = entries.into_iter().filter(|x| { predicate_filter(&x, &param) });

    for e in entries{
        println!("entry {:?}", e);
    }
}
