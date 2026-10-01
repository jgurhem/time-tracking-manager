use std::{collections::HashMap, error::Error, rc::Rc, sync::Mutex};

use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use gloo::{events::EventListener, timers::future::TimeoutFuture};
use wasm_bindgen::{
    convert::FromWasmAbi, describe::WasmDescribe, prelude::wasm_bindgen, JsCast, JsValue,
};

use crate::{
    args::Args,
    entries::Work,
    provider_handle::ProviderHandle,
    tablers::{MyTable, Table},
    utils::end_of_month,
};

use web_sys::{
    console::log_1, Document, Element, Event, EventInit, EventTarget, HtmlButtonElement,
    HtmlDivElement, HtmlElement, HtmlInputElement, HtmlLiElement, HtmlOptionElement,
    HtmlSelectElement, HtmlSpanElement, HtmlStyleElement, HtmlUListElement,
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

/// A row of the time table to export: the name displayed in Progessi and the
/// value (in days) for each day of the month.
struct Line {
    name: String,
    days: HashMap<u32, f64>,
}

fn sleep(ms: u32) -> TimeoutFuture {
    TimeoutFuture::new(ms)
}

/// Progessi renders its timesheet with Vue, asynchronously: poll until the
/// element is available.
async fn wait_for(document: &Document, selector: &str) -> Option<Element> {
    for _ in 0..200 {
        if let Some(element) = document.query_selector(selector).unwrap() {
            return Some(element);
        }
        sleep(50).await;
    }
    None
}

fn dispatch(target: &EventTarget, name: &str) {
    let init = EventInit::new();
    init.set_bubbles(true);
    let event =
        Event::new_with_event_init_dict(name, &init).expect("Event should be created successfully");
    let _ = target.dispatch_event(&event);
}

fn meta_selector(key: &str) -> String {
    format!(".tsv2-line-meta[data-line-key=\"{key}\"]")
}

fn get_line_keys(document: &Document) -> Vec<String> {
    document
        .query_selector_all(".tsv2-line-meta[data-line-key]")
        .expect("Lines should be available")
        .values()
        .into_iter()
        .filter_map(|e| {
            e.unwrap()
                .dyn_into::<Element>()
                .unwrap()
                .get_attribute("data-line-key")
        })
        .collect()
}

fn get_select(document: &Document, key: &str, class: &str) -> Option<HtmlSelectElement> {
    document
        .query_selector(&format!("{} select.{class}", meta_selector(key)))
        .unwrap()
        .map(|e| e.dyn_into::<HtmlSelectElement>().unwrap())
}

fn get_selected_text(select: &HtmlSelectElement) -> String {
    match usize::try_from(select.selected_index()) {
        Ok(i) => get_options_from_select(select)
            .get(i)
            .cloned()
            .unwrap_or_default(),
        Err(_) => String::new(),
    }
}

/// Name of a line: selected project followed by the selected phase, if any.
/// Empty when no project is selected.
fn get_line_name(document: &Document, key: &str) -> String {
    let Some(project) = get_select(document, key, "proj__select") else {
        return String::new();
    };
    if project.value().is_empty() {
        return String::new();
    }

    let mut name = get_selected_text(&project);
    if let Some(phase) = get_select(document, key, "tsv2-phase-select") {
        name += " ";
        name += &get_selected_text(&phase);
    }
    name
}

fn get_missing_lines(document: &Document, names: &[String]) -> Vec<String> {
    let mut names = names.to_owned();

    for key in get_line_keys(document) {
        let name = get_line_name(document, &key);
        if let Some(i) = names.iter().position(|n| name.contains(&n.to_lowercase())) {
            names.remove(i);
        }
    }
    names
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

/// Select the first option containing `val` and notify the page. Returns the
/// selected option text, empty if no option matches.
fn select_option(select: &HtmlSelectElement, val: &str) -> String {
    let val = val.to_lowercase();
    for (i, s) in get_options_from_select(select).iter().enumerate() {
        if s.contains(&val) {
            select.set_selected_index(i.try_into().unwrap());
            dispatch(select, "change");
            return s.clone();
        }
    }
    String::new()
}

/// Return the key of a line without project, adding one if needed.
async fn get_empty_line(document: &Document) -> Option<String> {
    let keys = get_line_keys(document);
    if let Some(key) = keys.iter().find(|k| get_line_name(document, k).is_empty()) {
        return Some(key.clone());
    }

    let button = document
        .query_selector(".tsv2-line-table__toolbar--top button.tsv2-btn--dash")
        .expect("add line button query was not valid")
        .expect("add line button was not found")
        .dyn_into::<HtmlButtonElement>()
        .expect("failed to cast to button");

    if !button
        .text_content()
        .unwrap_or_default()
        .contains("Ajouter une ligne")
    {
        panic!("We should find the button called 'Ajouter une ligne'")
    }

    button.click();

    for _ in 0..40 {
        sleep(50).await;
        if let Some(key) = get_line_keys(document)
            .into_iter()
            .find(|k| !keys.contains(k))
        {
            return Some(key);
        }
    }
    None
}

async fn add_lines(document: &Document, names: &[String]) {
    for val in names {
        let Some(key) = get_empty_line(document).await else {
            log!("failed to add a line for {}", val);
            continue;
        };

        let project =
            get_select(document, &key, "proj__select").expect("Project select should be available");

        let selected = select_option(&project, val);
        if !selected.is_empty() && !selected.contains("activité interne") {
            continue;
        }

        if selected.is_empty() {
            select_option(&project, "activité interne");
        }

        let phase = wait_for(
            document,
            &format!("{} select.tsv2-phase-select", meta_selector(&key)),
        )
        .await
        .map(|e| e.dyn_into::<HtmlSelectElement>().unwrap());

        if phase
            .map(|p| select_option(&p, val))
            .unwrap_or_default()
            .is_empty()
        {
            log!("time line not found for {}", val);
        }
    }
}

fn fill_line(document: &Document, key: &str, days: &HashMap<u32, f64>) {
    let inputs = document
        .query_selector_all(&format!(
            "input.tsv2-line-cell__input[data-line-key=\"{key}\"]"
        ))
        .expect("Lines should have days");

    for input in inputs.values() {
        let input = input
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .expect("Day should be an input");

        let day = input
            .get_attribute("data-day-idx")
            .expect("Day should have an index")
            .parse::<u32>()
            .expect("Day index should be cast to integer")
            + 1;

        let value = days.get(&day).copied().unwrap_or_default();
        let current = input.value();
        if (value == 0.0 && current.is_empty()) || current == value.to_string() {
            continue;
        }

        input.set_value(&value.to_string());
        dispatch(&input, "input");
        dispatch(&input, "change");
        let _ = input.blur();
    }
}

async fn fill(document: Document, lines: Vec<Line>) {
    let names: Vec<String> = lines.iter().map(|l| l.name.clone()).collect();
    let missing = get_missing_lines(&document, &names);

    log!("missing {:?}", missing);
    add_lines(&document, &missing).await;

    for key in get_line_keys(&document) {
        let name = get_line_name(&document, &key);
        if name.is_empty() {
            continue;
        }
        if let Some(line) = lines.iter().find(|l| name.contains(&l.name.to_lowercase())) {
            fill_line(&document, &key, &line.days);
        }
    }
}

impl<'a> Exporter<'a> for Progessi {
    type Table
        = MyTable<u8>
    where
        Self: 'a;

    fn export(
        &mut self,
        table: &Self::Table,
        display: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        let lines = table
            .row_headers()
            .map(|h| Line {
                name: display
                    .get(h.to_string().as_str())
                    .unwrap_or(&h.to_string())
                    .clone(),
                days: (1..=31)
                    .filter_map(|day| {
                        let date = Utc
                            .with_ymd_and_hms(self.start.year(), self.start.month(), day, 0, 0, 0)
                            .single()?;
                        Some((day, f64::from(table.get(h.clone(), date)) / 100.0))
                    })
                    .collect(),
            })
            .collect();

        wasm_bindgen_futures::spawn_local(fill(self.document.clone(), lines));
        Ok(())
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
    type Table
        = MyTable<u8>
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
        row.append_child(&create_cell(&self.document, "Total"))
            .unwrap();

        let row_headers: Vec<Work> = table.row_headers().cloned().collect();

        for r in row_headers {
            let row = create_row(&self.document);
            preview.append_child(&row).unwrap();
            let cell = create_cell(
                &self.document,
                &display
                    .get(&r.to_string())
                    .unwrap_or(&r.to_string())
                    .to_lowercase(),
            );
            row.append_child(&cell).unwrap();

            for date in &col_headers {
                let span = create_cell(&self.document, &table.get(r.clone(), *date).to_string());
                row.append_child(&span).unwrap();
            }
            row.append_child(&create_cell(&self.document, &table.row_sum(&r).to_string()))
                .unwrap();
        }

        Ok(())
    }
}

#[allow(clippy::await_holding_lock)]
async fn download_entries(handle: Rc<Mutex<ProviderHandle>>) {
    let mut handle = handle.lock().unwrap();
    handle
        .download_entries()
        .await
        .expect("Failed to download entries");
    handle.process().expect("Failed to process entries");
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

fn parse_french_date(date: &str) -> DateTime<Utc> {
    let mut parts = date.split_whitespace();
    let first_word = parts.next().expect("Date should have a first word");
    let translated = match first_word.to_lowercase().as_str() {
        "janvier" => "January",
        "février" => "February",
        "mars" => "March",
        "avril" => "April",
        "mai" => "May",
        "juin" => "June",
        "juillet" => "July",
        "août" => "August",
        "septembre" => "September",
        "octobre" => "October",
        "novembre" => "November",
        "décembre" => "December",
        _ => first_word,
    };

    let date = std::iter::once(translated)
        .chain(parts)
        .collect::<Vec<_>>()
        .join(" ");

    let date = date + " 01";

    let date = NaiveDate::parse_from_str(&date, "%B %Y %d").expect("Date should be valid");

    let date = date.and_hms_opt(0, 0, 0).expect("Invalid added time");

    date.and_utc()
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

        let element = wait_for(
            &document,
            ".tsv2-line-table__toolbar--top .tsv2-line-table__toolbar-left",
        )
        .await
        .expect("element in which to add button was not found");

        let header = document
            .query_selector("span.header-text")
            .expect("Sheet header containing current month should be available")
            .expect("Sheet header containing current month should be available")
            .dyn_into::<HtmlSpanElement>()
            .expect("Element should be cast into span")
            .inner_text();

        let start = header
            .split("-")
            .nth(1)
            .expect("Should get date separated by - from page header")
            .trim();

        let start = parse_french_date(start);
        let end = end_of_month(&start);

        let args = Args { start, end, ..args };

        let mut handle = ProviderHandle::new(args.clone()).expect("Provider not found");
        handle
            .download_entries()
            .await
            .expect("Failed to download entries");
        handle.process().expect("Failed to process entries");

        let handle = Rc::new(Mutex::new(handle));

        let clone = Rc::clone(&handle);
        let on_click = EventListener::new(&dowload, "click", move |_event| {
            let clone = Rc::clone(&clone);
            wasm_bindgen_futures::spawn_local(download_entries(clone));
        });
        on_click.forget();

        let clone = Rc::clone(&handle);
        let progessi = Progessi {
            start,
            document: document.clone(),
        };
        let on_click = EventListener::new(&fill, "click", move |_event| {
            let handle = clone.lock().unwrap();
            handle
                .export(Box::new(progessi.clone()))
                .expect("Failed to export entries");
        });
        on_click.forget();

        let progessi = ProgessiPreview::new(start, document.clone());
        let clone = Rc::clone(&handle);
        let on_click = EventListener::new(&preview, "click", move |_event| {
            let handle = clone.lock().unwrap();
            handle
                .export(Box::new(progessi.clone()))
                .expect("Failed to export entries");
        });
        on_click.forget();

        element.append_child(&dowload).unwrap();
        element.append_child(&fill).unwrap();
        element.append_child(&preview).unwrap();

        ProgessiHandle {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_avril() {
        let date = parse_french_date("avril 2025");
        assert_eq!(date.day(), 1);
        assert_eq!(date.month(), 4);
        assert_eq!(date.year(), 2025);
    }

    #[test]
    fn parse_mai() {
        let date = parse_french_date("Mai 2025");
        assert_eq!(date.day(), 1);
        assert_eq!(date.month(), 5);
        assert_eq!(date.year(), 2025);
    }
}
