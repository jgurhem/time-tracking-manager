use std::error::Error;

use crate::entries::Entry;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait(?Send)]
pub trait Provider {
    async fn load(
        &mut self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Entry>, Box<dyn Error>>;
}

pub mod clockify;
pub mod ics;
