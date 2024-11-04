use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fmt::{Display, Formatter},
};

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

        let provider = args.provider.clone();

        match provider.as_str() {
            "Clockify" | "clockify" => Ok(ProviderHandle::from_provider(
                args,
                Box::new(Clockify::new(options)),
            )),
            _ => Err(ProviderNotFound),
        }
    }

    pub fn from_provider(args: Args, provider: Box<dyn Provider>) -> ProviderHandle {
        let mut display = HashMap::new();

        for d in args.display.iter() {
            let (k, v) = utils::split_eq(d).unwrap();
            display.insert(k.to_string(), v.to_string());
        }

        ProviderHandle {
            table: Default::default(),
            args,
            display,
            provider: RefCell::new(provider),
            entries: Default::default(),
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
            .filter(|x| predicate_filter(x, &param))
            .map(|x| renames.predicate_rename(x))
            .collect();

        self.table = Proportional::process(entries);
        Ok(())
    }

    pub fn export(
        &self,
        mut exporter: Box<dyn Exporter<Table = MyTable<u8>>>,
    ) -> Result<(), Box<dyn Error>> {
        exporter.export(&self.table, &self.display)?;
        Ok(())
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub struct ProviderNotFound;

impl Display for ProviderNotFound {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Could not find the specifed provider.")
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono::{DateTime, TimeDelta, TimeZone, Utc};

    use crate::entries;

    use super::*;

    struct TestProvider {}

    #[async_trait(?Send)]
    impl Provider for TestProvider {
        async fn load(
            &mut self,
            start: DateTime<Utc>,
            end: DateTime<Utc>,
        ) -> Result<Vec<entries::Entry>, Box<dyn Error>> {
            let mut entries = Vec::new();
            let delta = end - start;

            for day in 0..delta.num_days() {
                entries.push(Entry {
                    billable: true,
                    description: String::from("my first entry"),
                    id: String::from("1"),
                    project: String::from("Project1"),
                    task: String::from(""),
                    start: start
                        .checked_add_signed(TimeDelta::hours(day * 24))
                        .unwrap(),
                    end: start
                        .checked_add_signed(TimeDelta::hours(day * 24 + 1))
                        .unwrap(),
                    tags: Default::default(),
                });
                entries.push(Entry {
                    billable: true,
                    description: String::from("my second entry"),
                    id: String::from("2"),
                    project: String::from("Project1"),
                    task: String::from("task 1"),
                    start: start
                        .checked_add_signed(TimeDelta::hours(day * 24 + 1))
                        .unwrap(),
                    end: start
                        .checked_add_signed(TimeDelta::hours(day * 24 + 2))
                        .unwrap(),
                    tags: Default::default(),
                });
                entries.push(Entry {
                    billable: true,
                    description: String::from("my third entry"),
                    id: String::from("3"),
                    project: String::from("Project2"),
                    task: String::from("task"),
                    start: start
                        .checked_add_signed(TimeDelta::hours(day * 24 + 2))
                        .unwrap(),
                    end: start
                        .checked_add_signed(TimeDelta::hours(day * 24 + 3))
                        .unwrap(),
                    tags: Default::default(),
                });
            }

            Ok(entries)
        }
    }

    #[derive(Debug, Default)]
    struct TestExporter;

    impl<'a> Exporter<'a> for TestExporter {
        type Table = MyTable<u8>
        where
            Self: 'a;

        fn export(
            &mut self,
            _: &Self::Table,
            _: &HashMap<String, String>,
        ) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
    }

    #[test]
    fn not_found() {
        let error = ProviderHandle::new(Args {
            provider: String::from("DoesNotExist"),
            ..Default::default()
        })
        .err()
        .unwrap();
        assert_eq!(error, ProviderNotFound);
    }

    #[tokio::test]
    async fn from_provider() -> Result<(), Box<dyn std::error::Error>> {
        let day = Utc.with_ymd_and_hms(2024, 10, 12, 0, 0, 0).unwrap();
        let mut handle = ProviderHandle::from_provider(
            Args {
                start: day.checked_add_signed(TimeDelta::hours(12)).unwrap(),
                end: day.checked_add_signed(TimeDelta::hours(37)).unwrap(),
                ..Default::default()
            },
            Box::new(TestProvider {}),
        );
        handle.download_entries().await?;
        handle.process()?;
        let exporter = Box::new(TestExporter);
        handle.export(exporter)?;
        Ok(())
    }
}
