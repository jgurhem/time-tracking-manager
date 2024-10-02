use std::collections::HashMap;

use chrono::{DateTime, Datelike, TimeZone, Utc};
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

use web_sys::console::log_1;

use super::Exporter;

#[wasm_bindgen]
pub struct Progessi {
    table: MyTable<u8>,
    display: HashMap<String, String>,
    args: Args,
}

impl<'a> Exporter<'a> for Progessi {
    type Table = MyTable<u8>
    where
        Self: 'a;

    fn export(table: &Self::Table, _: &HashMap<String, String>) {
        let months = table.group_by_month();
    }
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub struct PArgs {
    token: String,
    ignored: bool,
    billable: bool,
    ignore_list: Vec<String>,
    rename: Vec<String>,
    display: Vec<String>,
}

#[wasm_bindgen]
impl PArgs {
    pub fn new(
        token: String,
        ignored: bool,
        billable: bool,
        ignore_list: Vec<String>,
        rename: Vec<String>,
        display: Vec<String>,
    ) -> PArgs {
        PArgs {
            token,
            ignored,
            billable,
            ignore_list,
            rename,
            display,
        }
    }

}

impl PArgs {
    pub fn convert(&self
    ) -> Args {
        Args {
            token : self.token.clone(),
            ignored : self.ignored,
            billable: self.billable,
            ignore_list: self.ignore_list.clone(),
            rename: self.rename.clone(),
            display: self.display.clone(),
            ..Default::default()
        }
    }
}

#[wasm_bindgen]
impl Progessi {
    pub fn new(args: PArgs) -> Progessi {
        console_error_panic_hook::set_once();
        Progessi {
            table: MyTable::new(),
            display: HashMap::new(),
            args : args.convert(),
        }
    }

    pub async fn download(&mut self) {
        let mut c = Clockify::new(self.args.token.as_str());
        let entries = c.load(self.args.start, self.args.end).await.unwrap();

        let param = FilterParam::build(&self.args);
        let renames = Renames::build(&self.args).unwrap();
        let entries: Vec<Entry> = entries
            .into_iter()
            .filter(|x| predicate_filter(&x, &param))
            .map(|x| renames.predicate_rename(x))
            .collect();

        for e in &entries {
            log!("{:?}", e);
        }

        self.table = Proportional::process(entries);

        for d in self.args.display.iter() {
            let (k, v) = utils::split_eq(d).unwrap();
            self.display.insert(k.to_string(), v.to_string());
        }
    }

    pub fn row_headers(&self) -> Vec<String> {
        self.table.row_headers().cloned().collect()
    }

    pub fn get(&self, row:String, day:u32) -> u8 {
        let day = Utc.with_ymd_and_hms(self.args.start.year(), self.args.start.month(), day, 0, 0, 0).unwrap();
        self.table.get(row, day)
    }
}
