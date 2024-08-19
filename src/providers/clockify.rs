use super::Provider;
use crate::entries;
use chrono::DateTime;
use chrono::Utc;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde_json;
use std::error::Error;

#[derive(Debug)]
pub struct Clockify {
    token: String,
}

impl Clockify {
    pub fn new(token: String) -> Clockify {
        Clockify { token }
    }
}

#[derive(Deserialize, Debug)]
struct Workspace {
    id: String,
}

#[derive(Deserialize, Debug)]
struct User {
    id: String,
}

#[derive(Deserialize, Debug)]
struct Entry {
    id: String,
    description: String,
    billable: bool,
    project: Project,
    #[serde(rename = "timeInterval")]
    time_interval: TimeInterval,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    task: Task,
}

impl Entry {
    fn convert(&self) -> entries::Entry {
        entries::Entry {
            id: self.id.clone(),
            description: self.description.clone(),
            billable: self.billable,
            project: self.project.name.clone(),
            task: self.task.name.clone(),
            tags: self.tags.clone(),
            end: DateTime::parse_from_rfc3339(&self.time_interval.end)
                .unwrap()
                .to_utc(),
            start: DateTime::parse_from_rfc3339(&self.time_interval.start)
                .unwrap()
                .to_utc(),
        }
    }
}

#[derive(Deserialize, Debug)]
struct Project {
    name: String,
}

#[derive(Deserialize, Debug)]
struct TimeInterval {
    start: String,
    end: String,
}

#[derive(Deserialize, Debug, Default)]
struct Task {
    #[serde(default)]
    name: String,
}

impl Provider for Clockify {
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
        headers.append("X-Api-Key", HeaderValue::from_str(&self.token)?);

        let base = "https://api.clockify.me/api/v1";
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let req = client.get(format!("{base}/workspaces")).build()?;
        let workspace = client.execute(req).await?.json::<Vec<Workspace>>().await?;
        let workspace = &workspace.first().unwrap().id;

        let req = client.get(format!("{base}/user")).build()?;
        let user = client.execute(req).await?.json::<User>().await?.id;

        let format = "%Y-%m-%dT%H:%M:%SZ";
        let start = start.format(&format).to_string();
        let end = end.format(&format).to_string();

        let mut page = 1;
        loop {
            let req = client.get(format!("{base}/workspaces/{workspace}/user/{user}/time-entries?start={start}&end={end}&hydrated=true&page={page}&page-size=1")).build()?;
            let body = client.execute(req).await?.text().await?;
            // remove task:null that serde is not able to parse
            let body = body.replace("\"task\":null,", "");
            if body.is_empty() {
                break;
            }
            let res: Vec<Entry> = serde_json::from_str(&body).unwrap();
            for e in res {
                entries.push(e.convert());
            }
            page += 1;
        }

        Ok(entries)
    }
}
