use std::error::Error;

use chrono::{DateTime, Utc};

use crate::entries::Entry;

pub trait Provider {
    fn load(
        &mut self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> impl std::future::Future<Output = Result<Vec<Entry>, Box<dyn Error>>> + Send;
}

pub mod clockify;