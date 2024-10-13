use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fmt::{Display, Formatter},
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    args::Args,
    entries::Entry,
    exporters::Exporter,
    filters::{predicate_filter, FilterParam},
    providers::{clockify::Clockify, Provider},
    renamers::Renames,
    tablers::{proportional::Proportional, MyTable, Tabler},
    utils::{self, split_eq},
};

struct NotImplementedProvider;

#[async_trait(?Send)]
impl Provider for NotImplementedProvider {
    async fn load(
        &mut self,
        _: DateTime<Utc>,
        _: DateTime<Utc>,
    ) -> Result<Vec<Entry>, Box<dyn Error>> {
        todo!()
    }
}

pub struct ProviderHandle {
    provider: RefCell<Box<dyn Provider>>,
    args: Args,
    display: HashMap<String, String>,
    table: MyTable<u8>,
    entries: Vec<Entry>,
}

impl ProviderHandle {
    pub fn new(args: Args) -> Result<ProviderHandle, ProviderNotFound> {
        let options = args
            .provider_options
            .clone()
            .into_iter()
            .map(|o| split_eq(&o).unwrap())
            .collect();

        let mut display = HashMap::new();

        for d in args.display.iter() {
            let (k, v) = utils::split_eq(d).unwrap();
            display.insert(k.to_string(), v.to_string());
        }

        let provider = args.provider.clone();

        let handle = ProviderHandle {
            table: Default::default(),
            args,
            display,
            provider: RefCell::new(Box::new(NotImplementedProvider {})),
            entries: Default::default(),
        };

        match provider.as_str() {
            "Clockify" | "clockify" => Ok(ProviderHandle {
                provider: RefCell::new(Box::new(Clockify::new(options))),
                ..handle
            }),
            _ => Err(ProviderNotFound),
        }
    }

    pub async fn download_entries(&mut self) -> Result<(), Box<dyn Error>> {
        let mut provider = self.provider.borrow_mut();
        self.entries = provider.load(self.args.start, self.args.end).await?;
        Ok(())
    }

    pub fn process(&mut self) -> Result<(), Box<dyn Error>> {
        let param = FilterParam::build(&self.args);
        let renames = Renames::build(&self.args)?;
        let entries = self
            .entries
            .clone()
            .into_iter()
            .filter(|x| predicate_filter(&x, &param))
            .map(|x| renames.predicate_rename(x))
            .collect();

        self.table = Proportional::process(entries);
        Ok(())
    }

    pub fn export(
        &self,
        exporter: Box<dyn Exporter<Table = MyTable<u8>>>,
    ) -> Result<(), Box<dyn Error>> {
        exporter.export(&self.table, &self.display)?;
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub struct ProviderNotFound;

impl Display for ProviderNotFound {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Could not find the specifed provider.")
    }
}
