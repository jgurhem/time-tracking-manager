use std::{collections::HashMap, error::Error, str::FromStr};

use async_trait::async_trait;
use chrono::{Datelike, NaiveDateTime, TimeZone, Utc};
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
    utils::{self, end_of_month},
};

use std::sync::Arc;
use std::sync::Mutex;
use web_sys::{
    console::log_1, Document, Event, HtmlButtonElement, HtmlDivElement, HtmlInputElement,
    HtmlOptionElement, HtmlSelectElement, NodeList,
};

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

fn get_selected_from_timeline(timeline: &HtmlDivElement) -> String {
    let selects = timeline
        .query_selector_all("select")
        .expect("Timeline should contain selects");

    let mut selected = String::new();

    for s in selects.values() {
        let s = s.unwrap().dyn_into::<HtmlSelectElement>().unwrap();
        let options = s.query_selector_all("option").unwrap();

        selected += options
            .get(s.selected_index().try_into().unwrap())
            .unwrap()
            .dyn_into::<HtmlOptionElement>()
            .unwrap()
            .text()
            .as_str();
    }
    selected
}

fn get_timelines(document: &Document) -> NodeList {
    document
        .query_selector_all(".fc-timeline")
        .expect("Timelines should be available")
}

fn get_missing_timelines(
    timelines: &NodeList,
    rows: &Vec<String>,
    display: &HashMap<String, String>,
) -> Vec<String> {
    let mut rows = rows.clone();

    for i in 0..rows.len() {
        let row = &rows[i];
        rows[i] = display.get(row).unwrap_or(row).to_lowercase();
    }

    for timeline in timelines.values() {
        let timeline = timeline
            .expect("Should get a timeline")
            .dyn_into::<HtmlDivElement>()
            .expect("Timeline should be a div element");
        let name = get_selected_from_timeline(&timeline).to_lowercase();
        for i in 0..rows.len() {
            if name.contains(&rows[i]) {
                rows.remove(i);
                break;
            }
        }
    }
    rows
}

fn get_options_from_select(select: &HtmlSelectElement) -> Vec<String> {
    select
        .query_selector_all("option")
        .unwrap()
        .values()
        .into_iter()
        .map(|e| {
            e.unwrap()
                .dyn_into::<HtmlOptionElement>()
                .unwrap()
                .text()
                .to_lowercase()
        })
        .collect()
}

fn add_timelines(document: &Document, timelines: &Vec<String>) {
    let element = document
        .query_selector(".fc-addcontrol")
        .expect("element containing add timeline button was not found")
        .expect("element containing add timeline button was not found")
        .dyn_into::<HtmlDivElement>()
        .expect("element should be a div element")
        .query_selector("button")
        .expect("timeline button was not found")
        .expect("timeline button was not found")
        .dyn_into::<HtmlButtonElement>()
        .expect("failed to cast to button");

    if Some("Ajouter une ligne".to_string()) != element.text_content() {
        panic!("We should find the button called 'Ajouter une ligne'")
    }

    for val in timelines {
        element.click();

        // get first selector without value then get the timeline from it
        let timeline = document
            .query_selector("div.table-cell-workeffort > select:nth-child(1) > option[value=\"?\"]")
            .unwrap()
            .unwrap()
            .closest(".fc-timeline")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlDivElement>()
            .expect("Timeline should be a div element");

        let selects = timeline
            .query_selector_all("div.table-cell-workeffort > select")
            .unwrap();

        let s0 = selects
            .get(0)
            .expect("First select should be available")
            .dyn_into::<HtmlSelectElement>()
            .expect("Node should be a select");

        let o0 = get_options_from_select(&s0);
        let mut selected = String::new();
        for i in 0..o0.len() {
            if o0[i].contains(&val.to_lowercase()) {
                s0.set_selected_index(i.try_into().unwrap());
                let event = Event::new("change").expect("Event should be created successfully");
                let _ = s0.dispatch_event(&event);
                selected = o0[i].clone();
                break;
            }
        }

        if !selected.is_empty() && !selected.contains("Activité interne") {
            continue;
        }

        if selected.is_empty() {
            let s1 = selects
                .get(1)
                .expect("First select should be available")
                .dyn_into::<HtmlSelectElement>()
                .expect("Node should be a select");

            let o1 = get_options_from_select(&s1);
            let mut selected = String::new();
            for i in 0..o1.len() {
                if o1[i].contains(&val.to_lowercase()) {
                    s1.set_selected_index(i.try_into().unwrap());
                    let event = Event::new("change").expect("Event should be created successfully");
                    let _ = s1.dispatch_event(&event);
                    selected = o1[i].clone();
                    break;
                }
            }

            if selected.is_empty() {
                log!("time line not found for {}", val);
            }
        }
    }
}

impl<'a> Exporter<'a> for Progessi {
    type Table = MyTable<u8>
    where
        Self: 'a;

    fn export(&self, _: &Self::Table, _: &HashMap<String, String>) -> Result<(), Box<dyn Error>> {
        let timelines = get_timelines(&self.document);

        let row_headers: Vec<String> = self.table.row_headers().cloned().collect();
        let missing = get_missing_timelines(&timelines, &row_headers, &self.display);

        log!("missing {:?}", missing);
        add_timelines(&self.document, &missing);

        let timelines = get_timelines(&self.document);
        for timeline in timelines.values() {
            let timeline = timeline
                .expect("Should get a timeline")
                .dyn_into::<HtmlDivElement>()
                .expect("Timeline should be a div element");

            let name = get_selected_from_timeline(&timeline).to_lowercase();

            for h in &row_headers {
                if name.contains(&self.display.get(h).unwrap_or(h).to_lowercase()) {
                    let days = timeline
                        .query_selector_all(".dayparent")
                        .expect("Timelines should have days");

                    for day in days.values() {
                        let day = day
                            .expect("Should get a day")
                            .dyn_into::<HtmlDivElement>()
                            .expect("Day should be a div element");

                        let header = day
                            .query_selector(".day-numbers")
                            .expect("Day should have a header")
                            .expect("Day should have a header")
                            .dyn_into::<HtmlDivElement>()
                            .expect("Day header should be an div")
                            .text_content()
                            .expect("Day header should have text")
                            .parse::<u32>()
                            .expect("Day number should be cast to integer");

                        let input = day
                            .query_selector("input")
                            .expect("Day should have an input")
                            .expect("Day should have an input")
                            .dyn_into::<HtmlInputElement>()
                            .expect("Day should be an input");

                        let mut value: f64 = self.get(h.clone(), header).into();
                        value /= 100.0;

                        input.set_value(value.to_string().as_str());
                        let event =
                            Event::new("change").expect("Event should be created successfully");
                        let _ = input.dispatch_event(&event);
                    }

                    break;
                }
            }
        }
        Ok(())
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
    pub async fn new(args: Args, document: JsValue) -> ProgessiHandle {
        console_error_panic_hook::set_once();
        let document = document
            .dyn_into::<Document>()
            .expect("input should be a document");

        let dowload = document
            .create_element("button")
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();
        dowload.set_text_content(Some("Refresh entries"));
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

        let start = document
            .query_selector(".menu-selector-validateTimesheet > form > input[name=\"fromDate\"]")
            .expect("Validate button should have a date input")
            .expect("Validate button should have a date input")
            .dyn_into::<HtmlInputElement>()
            .expect("Element should be cast into input")
            .value()
            .replace(" ", "T");

        let start = NaiveDateTime::from_str(&start)
            .expect("date should be valid")
            .and_local_timezone(Utc)
            .unwrap();
        let end = end_of_month(&start);

        let args = Args { start, end, ..args };

        let mut progessi = Progessi::new(args, document);
        progessi.download_entries().await;
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
