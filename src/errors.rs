use std::fmt::{Display, Formatter};

use thiserror::Error;

#[derive(Error, Debug)]
pub struct SplitError {
    pub field: String,
    pub reason: String,
}

impl Display for SplitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to split {} due to: {}", self.field, self.reason)
    }
}
