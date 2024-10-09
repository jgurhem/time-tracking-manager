use std::{
    cell::RefCell,
    error::Error,
    fmt::{Display, Formatter},
};

use crate::{args::Args, entries::Entry};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use clockify::Clockify;

#[async_trait(?Send)]
pub trait Provider {
    async fn load(
        &mut self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Entry>, Box<dyn Error>>;
}

pub mod clockify;

pub struct ProviderHandle {
    pub provider: RefCell<Box<dyn Provider>>,
    pub args: Args,
}

impl ProviderHandle {
    pub fn new(provider: &str, args: Args) -> Result<ProviderHandle, ProviderNotFound> {
        match provider {
            "Clockify" | "clockify" => Ok(ProviderHandle {
                provider: RefCell::new(Box::new(Clockify::new(args.token.clone()))),
                args,
            }),
            _ => Err(ProviderNotFound),
        }
    }
}

#[derive(Debug)]
pub struct ProviderNotFound;

impl Display for ProviderNotFound {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Could not find the specifed provider.")
    }
}

impl Error for ProviderNotFound {}
