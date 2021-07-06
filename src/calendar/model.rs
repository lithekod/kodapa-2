//! Rust-representations of common Google Calendar API types.

use serde::{Deserialize, Serialize};

pub mod events;

#[derive(Debug)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timestamp {
    date: Option<String>,
    date_time: Option<String>,
    end_time_unspecified: Option<bool>,
}
