use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{Datelike, TimeZone, Utc};
use gloo::events::EventListener;
use wasm_bindgen::{
    convert::FromWasmAbi, describe::WasmDescribe, prelude::wasm_bindgen, JsCast, JsValue,
};

use crate::{
    args::Args,
    entries::Entry,
    filters::{predicate_filter, FilterParam},
    providers::ProviderHandle,
    renamers::Renames,
    tablers::{proportional::Proportional, MyTable, Table, Tabler},
    utils,
};

use std::sync::Arc;
use std::sync::Mutex;
use web_sys::{console::log_1, Document, HtmlButtonElement, HtmlDivElement};

use super::{Exporter, WebExporter};

pub struct Progessi {
    table: MyTable<u8>,
    display: HashMap<String, String>,
    document: Document,
    provider: ProviderHandle,
}

#[wasm_bindgen]
pub struct ProgessiHandle {
    progessi: Arc<Mutex<Progessi>>,
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

impl<'a> Exporter<'a> for Progessi {
    type Table = MyTable<u8>
    where
        Self: 'a;

    fn export(&self, table: &Self::Table, _: &HashMap<String, String>) {
        let months = table.group_by_month();
    }
}

#[async_trait(?Send)]
impl<'a> WebExporter<'a> for Progessi {
    async fn download_entries(&mut self) {
        let args = &self.provider.args;
        let provider = &self.provider.provider;
        let entries = provider
            .borrow_mut()
            .load(args.start, args.end)
            .await
            .unwrap();

        let param = FilterParam::build(&args);
        let renames = Renames::build(&args).unwrap();
        let entries: Vec<Entry> = entries
            .into_iter()
            .filter(|x| predicate_filter(&x, &param))
            .map(|x| renames.predicate_rename(x))
            .collect();

        for e in &entries {
            log!("{:?}", e);
        }

        self.table = Proportional::process(entries);

        for d in args.display.iter() {
            let (k, v) = utils::split_eq(d).unwrap();
            self.display.insert(k.to_string(), v.to_string());
        }
    }
}

impl Progessi {
    pub fn new(args: Args, document: Document) -> Progessi {
        Progessi {
            table: MyTable::new(),
            display: HashMap::new(),
            document,
            provider: ProviderHandle::new("clockify", args).expect("Provider not found"),
        }
    }

    pub fn row_headers(&self) -> Vec<String> {
        self.table.row_headers().cloned().collect()
    }

    pub fn get(&self, row: String, day: u32) -> u8 {
        let day = Utc
            .with_ymd_and_hms(
                self.provider.args.start.year(),
                self.provider.args.start.month(),
                day,
                0,
                0,
                0,
            )
            .unwrap();
        self.table.get(row, day)
    }
}

async fn download_entries(progessi: Arc<Mutex<Progessi>>) {
    progessi.lock().unwrap().download_entries().await;
}

#[wasm_bindgen]
impl ProgessiHandle {
    pub fn new(args: Args, document: JsValue) -> ProgessiHandle {
        console_error_panic_hook::set_once();
        let document = document
            .dyn_into::<Document>()
            .expect("input should be a document");

        let dowload = document
            .create_element("button")
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();
        dowload.set_text_content(Some("Download entries"));
        dowload.set_type("button");
        dowload.set_class_name("btn btn-primary btn-sm");

        let fill = document
            .create_element("button")
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();
        fill.set_text_content(Some("Fill time table"));
        fill.set_type("button");
        fill.set_class_name("btn btn-primary btn-sm");

        let element = document
            .query_selector(".fc-addcontrol")
            .expect("element in which to add button was not found")
            .expect("element in which to add button was not found")
            .dyn_into::<HtmlDivElement>()
            .expect("should be a div element");

        let progessi = Progessi::new(args, document);
        let progessi = Arc::new(Mutex::new(progessi));

        let clone = Arc::clone(&progessi);
        let on_click = EventListener::new(&dowload, "click", move |_event| {
            let clone = Arc::clone(&clone);
            wasm_bindgen_futures::spawn_local(download_entries(clone));
        });
        on_click.forget();

        let clone = Arc::clone(&progessi);
        let on_click = EventListener::new(&fill, "click", move |_event| {
            let progessi = clone.lock().unwrap();
            progessi.export(&progessi.table, &progessi.display);
        });
        on_click.forget();

        element.append_child(&dowload).unwrap();
        element.append_child(&fill).unwrap();

        ProgessiHandle { progessi }
    }
}
