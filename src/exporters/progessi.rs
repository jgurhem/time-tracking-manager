use std::collections::HashMap;

use chrono::{Datelike, TimeZone, Utc};
use gloo::events::EventListener;
use wasm_bindgen::{
    convert::FromWasmAbi, describe::WasmDescribe, prelude::wasm_bindgen, JsCast, JsValue,
};

use crate::{
    args::Args,
    entries::Entry,
    filters::{predicate_filter, FilterParam},
    providers::{clockify::Clockify, Provider},
    renamers::Renames,
    tablers::{proportional::Proportional, MyTable, Table, Tabler},
    utils,
};

use web_sys::{console::log_1, Document, HtmlButtonElement};

use super::Exporter;

#[wasm_bindgen]
pub struct Progessi {
    table: MyTable<u8>,
    display: HashMap<String, String>,
    args: Args,
    document: Document,
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

impl WasmDescribe for Args {
    fn describe() {
        <wasm_bindgen::JsValue as WasmDescribe>::describe();
    }
}

impl FromWasmAbi for Args {
    type Abi = <wasm_bindgen::JsValue as FromWasmAbi>::Abi;

    #[inline]
    unsafe fn from_abi(js: Self::Abi) -> Self {
        let js = JsValue::from_abi(js);
        serde_wasm_bindgen::from_value(js).unwrap()
    }
}

#[wasm_bindgen]
impl Progessi {
    pub fn new(args: Args, document: JsValue) -> Progessi {
        console_error_panic_hook::set_once();
        Progessi {
            table: MyTable::new(),
            display: HashMap::new(),
            args,
            document: document
                .dyn_into::<Document>()
                .expect("input should be a document"),
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

    pub fn get(&self, row: String, day: u32) -> u8 {
        let day = Utc
            .with_ymd_and_hms(
                self.args.start.year(),
                self.args.start.month(),
                day,
                0,
                0,
                0,
            )
            .unwrap();
        self.table.get(row, day)
    }

    pub fn button(&self) {
        let body = self
            .document
            .body()
            .expect("document expect to have have a body");

        let button = self
            .document
            .create_element("button")
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();
        button.set_text_content(Some("Download entries"));
        button.set_type("button");
        button.set_class_name("btn btn-primary btn-sm");

        let on_click = EventListener::new(&button, "click", move |_event| {
            log!("Hello World Gloo : WebAssemblyMan");
        });

        on_click.forget();
        body.append_child(&button).unwrap();
    }
}
