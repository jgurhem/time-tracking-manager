pub mod console;

use std::fmt::Display;

use serde::Serialize;

use crate::tablers::Table;

pub trait Exporter<'a> {
    type Table: Table<Item<'a>: Display + Serialize>
    where
        Self: 'a;
    fn export(table: &Self::Table);
}
