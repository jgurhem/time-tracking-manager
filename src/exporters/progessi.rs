use std::collections::HashMap;

use clap::Parser;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    args::Args,
    entries::Entry,
    filters::{predicate_filter, FilterParam},
    providers::{clockify::Clockify, Provider},
    renamers::Renames,
    tablers::{proportional::Proportional, MyTable, Table, Tabler},
    utils,
};

use super::Exporter;

#[wasm_bindgen]
pub struct Progessi {
    table: MyTable<u8>
}

impl<'a> Exporter<'a> for Progessi {
    type Table = MyTable<u8>
    where
        Self: 'a;

    fn export(table: &Self::Table, _: &HashMap<String, String>) {
        let months = table.group_by_month();
    }
}

#[wasm_bindgen]
impl Progessi {
    pub fn new() -> Progessi {
        Progessi { table: MyTable::new() }
    }

    pub async fn compute(&mut self) {
        let args = Args::parse();

        let mut c = Clockify::new(args.token.as_str());
        let entries = c.load(args.start, args.end).await.unwrap();

        let param = FilterParam::build(&args);
        let renames = Renames::build(&args).unwrap();
        let entries: Vec<Entry> = entries
            .into_iter()
            .filter(|x| predicate_filter(&x, &param))
            .map(|x| renames.predicate_rename(x))
            .collect();

        self.table = Proportional::process(entries);
        let mut display = HashMap::new();

        for d in args.display.iter() {
            let (k, v) = utils::split_eq(d).unwrap();
            display.insert(k.to_string(), v.to_string());
        }
    }
}