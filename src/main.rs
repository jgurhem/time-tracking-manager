use std::error::Error;
use time_tracking_manager::{
    args::parse_args,
    exporters::{aggregated::Aggregated, console::Console, csv::CSV, sunburstchart::SunburstChart},
    provider_handle::ProviderHandle,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args();
    dbg!(&args);

    let exporter_options = args.exporter_options.clone();
    let mut handle = ProviderHandle::new(args).expect("Provider should be available");
    handle.download_entries().await?;
    handle.process()?;

    handle.export(Box::new(Console::stdout_output())).unwrap();
    handle.export(Box::new(CSV {})).unwrap();
    handle.export(Box::new(SunburstChart {})).unwrap();
    handle.export(Box::new(Aggregated::stdout_output(&exporter_options)))?;

    Ok(())
}
