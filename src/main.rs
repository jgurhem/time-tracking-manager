use clap::Parser;
use std::error::Error;
use time_tracking_manager::{
    args::Args,
    exporters::{console::Console, csv::CSV},
    provider_handle::ProviderHandle,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    dbg!(&args);

    let mut handle = ProviderHandle::new(args).expect("Provider should be available");
    handle.download_entries().await?;
    handle.process()?;

    handle.export(Box::new(Console {})).unwrap();
    handle.export(Box::new(CSV {})).unwrap();

    Ok(())
}
