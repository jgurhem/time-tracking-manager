pub mod console;
pub mod csv;
pub mod progessi;

use std::{collections::HashMap, error::Error, fmt::Display};

use async_trait::async_trait;
use serde::Serialize;

use crate::tablers::Table;

pub trait Exporter<'a> {
    type Table: Table<Item<'a>: Display + Serialize>
    where
        Self: 'a;
    fn export(
        &self,
        table: &Self::Table,
        display: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>>;
}

#[async_trait(?Send)]
pub trait WebExporter<'a>: Exporter<'a> {
    async fn download_entries(&mut self);
}
