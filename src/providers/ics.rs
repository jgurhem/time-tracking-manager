use super::Provider;
use crate::entries::{self, Entry};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use std::{collections::HashMap, error::Error};

#[derive(Debug)]
pub struct Ics {
    url: String,
}

impl Ics {
    pub fn new(options: HashMap<String, String>) -> Ics {
        Ics {
            url: options
                .get("url")
                .expect("Ics provider options should contain a URL")
                .clone(),
        }
    }
}

#[async_trait(?Send)]
impl Provider for Ics {
    async fn load(
        &mut self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<entries::Entry>, Box<dyn Error>> {
        let mut entries: Vec<entries::Entry> = Vec::new();

        let mut headers = HeaderMap::new();
        headers.append(
            "content-type",
            HeaderValue::from_str("application/json").expect("Hard coded values should be valid"),
        );

        let client = Client::builder().default_headers(headers).build()?;

        let request = client.get(self.url.clone()).build()?;
        let response = client.execute(request).await?;
        let body = response.bytes().await?;
        let mut body = body.as_ref();

        let parser = ical::IcalParser::new(&mut body);

        for calendar in parser {
            let calendar = calendar?;

            for event in calendar.events {
                let mut event_name = None;
                let mut event_description = None;
                let mut event_start = None;
                let mut event_end = None;

                for property in event.properties {
                    let date_format = "%Y%m%dT%H%M%S";
                    match (property.name.as_str(), property.value) {
                        ("SUMMARY", value) => {
                            event_name = value;
                        }
                        ("DESCRIPTION", value) => {
                            event_description = value;
                        }
                        ("DTSTART", Some(value)) => {
                            event_start =
                                Some(NaiveDateTime::parse_from_str(&value, date_format)?.and_utc());
                        }
                        ("DTEND", Some(value)) => {
                            event_end =
                                Some(NaiveDateTime::parse_from_str(&value, date_format)?.and_utc());
                        }
                        _ => {}
                    }
                }

                let (Some(event_name), Some(event_start), Some(event_end)) =
                    (event_name, event_start, event_end)
                else {
                    continue;
                };

                if event_end < start || event_start > end {
                    continue;
                }

                let mut workday = true;
                let mut absolute = None;

                if let Some(description) = &event_description {
                    for line in description.lines() {
                        let (key, value) = line.split_once(':').unwrap_or((line, ""));
                        let key = key.trim_ascii();
                        let value = value.trim_ascii();

                        match (key, value) {
                            ("absolute", duration) => {
                                absolute =
                                    Some(duration.parse().expect("Invalid absolute duration"));
                            }
                            ("workday" | "work-day", "0" | "no" | "false") => workday = false,
                            ("outofoffice" | "out-of-office", "" | "1" | "yes" | "true") => {
                                workday = false
                            }
                            _ => {}
                        }
                    }
                }

                let (project, task) = event_name.split_once(':').unwrap_or((&event_name, ""));
                let project = project.trim_ascii();
                let task = task.trim_ascii();

                entries.push(Entry {
                    id: event_name.clone(),
                    description: event_description.unwrap_or(event_name.clone()),
                    billable: true,
                    project: project.to_string(),
                    task: task.to_string(),
                    tags: if workday {
                        vec![]
                    } else {
                        vec![String::from("out-of-office")]
                    },
                    end: event_end,
                    start: event_start,
                    absolute,
                });
            }
        }

        Ok(entries)
    }
}
