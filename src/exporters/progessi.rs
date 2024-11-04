use std::{collections::HashMap, error::Error, str::FromStr};

use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Utc};
use gloo::events::EventListener;
use wasm_bindgen::{
    convert::FromWasmAbi, describe::WasmDescribe, prelude::wasm_bindgen, JsCast, JsValue,
    UnwrapThrowExt,
};

use crate::{
    args::Args,
    provider_handle::ProviderHandle,
    tablers::{MyTable, Table},
    utils::end_of_month,
};

use std::sync::Arc;
use std::sync::Mutex;
use web_sys::{
    console::log_1, Document, Event, HtmlButtonElement, HtmlDivElement, HtmlElement,
    HtmlInputElement, HtmlLiElement, HtmlOptionElement, HtmlSelectElement, HtmlStyleElement,
    HtmlUListElement, NodeList,
};

use super::Exporter;

#[derive(Clone)]
pub struct Progessi {
    pub start: DateTime<Utc>,
    pub document: Document,
}

#[wasm_bindgen]
pub struct ProgessiHandle {}

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
    rows: &[String],
    display: &HashMap<String, String>,
) -> Vec<String> {
    let mut rows = rows.to_owned();

    for row in &mut rows {
        *row = display.get(row).unwrap_or(row).to_lowercase();
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
        for (i, s) in o0.iter().enumerate() {
            if s.contains(&val.to_lowercase()) {
                s0.set_selected_index(i.try_into().unwrap());
                let event = Event::new("change").expect("Event should be created successfully");
                let _ = s0.dispatch_event(&event);
                selected = s.clone();
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
            for (i, s) in o1.iter().enumerate() {
                if s.contains(&val.to_lowercase()) {
                    s1.set_selected_index(i.try_into().unwrap());
                    let event = Event::new("change").expect("Event should be created successfully");
                    let _ = s1.dispatch_event(&event);
                    selected = s.clone();
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

    fn export(
        &mut self,
        table: &Self::Table,
        display: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        let timelines = get_timelines(&self.document);

        let row_headers: Vec<String> = table.row_headers().cloned().collect();
        let missing = get_missing_timelines(&timelines, &row_headers, display);

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
                if name.contains(&display.get(h).unwrap_or(h).to_lowercase()) {
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

                        let mut value: f64 = self.get(table, h.clone(), header).into();
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

impl Progessi {
    pub fn get(&self, table: &MyTable<u8>, row: String, day: u32) -> u8 {
        let day = Utc
            .with_ymd_and_hms(self.start.year(), self.start.month(), day, 0, 0, 0)
            .unwrap();
        table.get(row, day)
    }
}

#[derive(Clone)]
pub struct ProgessiPreview {
    start: DateTime<Utc>,
    document: Document,
}

impl ProgessiPreview {
    pub fn new(start: DateTime<Utc>, document: Document) -> ProgessiPreview {
        let head = document.head().unwrap();
        let style = document
            .create_element("style")
            .unwrap()
            .dyn_into::<HtmlStyleElement>()
            .unwrap();

        style.set_type("text/css");
        style.set_text_content(Some(
            r"
            #ttm-preview {
                display: table;
                border-collapse: collapse;
              }

            #ttm-row {
                display: table-row;
            }

            #ttm-cell {
                display: table-cell;
                padding: 5px;
                border: 1px solid #ccc;
                text-align: center;
            }
            ",
        ));
        head.append_child(&style).unwrap();

        ProgessiPreview { start, document }
    }
}

fn create_cell(document: &Document, text: &str) -> HtmlElement {
    let cell = document
        .create_element("div")
        .unwrap()
        .dyn_into::<HtmlDivElement>()
        .unwrap();
    cell.set_text_content(Some(text));
    cell.set_id("ttm-cell");

    cell.into()
}

fn create_row(document: &Document) -> HtmlElement {
    let row = document
        .create_element("li")
        .unwrap()
        .dyn_into::<HtmlLiElement>()
        .unwrap();
    row.set_id("ttm-row");

    row.into()
}

impl<'a> Exporter<'a> for ProgessiPreview {
    type Table = MyTable<u8>
    where
        Self: 'a;

    fn export(
        &mut self,
        table: &Self::Table,
        display: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(element) = self
            .document
            .query_selector("#ttm-preview")
            .expect("old preview query was not valid")
        {
            element.remove();
        }

        let element = self
            .document
            .query_selector("#TIMESHEET_MESSAGE")
            .expect("element in which to add preview was not found")
            .expect("element in which to add preview was not found")
            .dyn_into::<HtmlDivElement>()
            .expect("should be a div element");

        let preview = self
            .document
            .create_element("ul")
            .unwrap()
            .dyn_into::<HtmlUListElement>()
            .unwrap();
        preview.set_id("ttm-preview");
        element.append_child(&preview).unwrap();

        let row = create_row(&self.document);
        preview.append_child(&row).unwrap();

        let cell = create_cell(
            &self.document,
            &(self.start.year().to_string() + "/" + &self.start.month().to_string()),
        );
        row.append_child(&cell).unwrap();

        let mut col_headers: Vec<DateTime<Utc>> = table.col_headers().cloned().collect();
        col_headers.sort();

        for date in &col_headers {
            let cell = create_cell(&self.document, &date.day().to_string());
            row.append_child(&cell).unwrap();
        }

        let row_headers: Vec<String> = table.row_headers().cloned().collect();

        for r in row_headers {
            let row = create_row(&self.document);
            preview.append_child(&row).unwrap();
            let cell = create_cell(
                &self.document,
                &display.get(&r).unwrap_or(&r).to_lowercase(),
            );
            row.append_child(&cell).unwrap();

            for date in &col_headers {
                let span = create_cell(
                    &self.document,
                    &table.get(r.clone(), *date).to_string(),
                );
                row.append_child(&span).unwrap();
            }
        }

        Ok(())
    }
}

async fn download_entries(handle: Arc<Mutex<ProviderHandle>>) {
    handle
        .lock()
        .unwrap()
        .download_entries()
        .await
        .unwrap_throw();
    handle.lock().unwrap().process().unwrap_throw();
}

fn create_button(document: &Document, text: &str) -> HtmlButtonElement {
    let button = document
        .create_element("button")
        .unwrap()
        .dyn_into::<HtmlButtonElement>()
        .unwrap();
    button.set_text_content(Some(text));
    button.set_type("button");
    button.set_class_name("btn btn-primary btn-sm");
    button
}

#[wasm_bindgen]
impl ProgessiHandle {
    pub async fn new(args: Args, document: JsValue) -> ProgessiHandle {
        console_error_panic_hook::set_once();
        let document = document
            .dyn_into::<Document>()
            .expect("input should be a document");

        let dowload = create_button(&document, "Refresh entries");
        let fill = create_button(&document, "Fill time table");
        let preview = create_button(&document, "Preview");

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

        let mut handle = ProviderHandle::new(args.clone()).expect("Provider not found");
        handle.download_entries().await.unwrap_throw();
        handle.process().unwrap_throw();

        let handle = Arc::new(Mutex::new(handle));

        let clone = Arc::clone(&handle);
        let on_click = EventListener::new(&dowload, "click", move |_event| {
            let clone = Arc::clone(&clone);
            wasm_bindgen_futures::spawn_local(download_entries(clone));
        });
        on_click.forget();

        let clone = Arc::clone(&handle);
        let progessi = Progessi {
            start,
            document: document.clone(),
        };
        let on_click = EventListener::new(&fill, "click", move |_event| {
            let handle = clone.lock().unwrap();
            handle.export(Box::new(progessi.clone())).unwrap_throw();
        });
        on_click.forget();

        let progessi = ProgessiPreview::new(start, document.clone());
        let clone = Arc::clone(&handle);
        let on_click = EventListener::new(&preview, "click", move |_event| {
            let handle = clone.lock().unwrap();
            handle.export(Box::new(progessi.clone())).unwrap_throw();
        });
        on_click.forget();

        element.append_child(&dowload).unwrap();
        element.append_child(&fill).unwrap();
        element.append_child(&preview).unwrap();

        ProgessiHandle {}
    }
}
