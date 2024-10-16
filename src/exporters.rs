pub mod console;
pub mod csv;
pub mod progessi;

use std::{collections::HashMap, error::Error, fmt::Display};

use serde::Serialize;

use crate::tablers::Table;

pub trait Exporter<'a> {
    type Table: Table<Item<'a>: Display + Serialize>
    where
        Self: 'a;
    fn export(
        &mut self,
        table: &Self::Table,
        display: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>>;
}
