//! Rust-representations of the Events Google Calendar API.
//!
//! See `https://developers.google.com/calendar/api/v3/reference/events`.

use serde::{Deserialize, Serialize};

use super::Timestamp;

#[derive(Debug)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventsListRequest {
    calendar_id: String,
    page_token: Option<String>,
    show_deleted: Option<bool>,
    single_events: Option<bool>,
    time_max: Option<String>,
    time_min: Option<String>,
}

#[derive(Debug)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventsListResponse {
    items: Vec<Event>,
    next_page_token: Option<String>,
}

#[derive(Debug)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    start: Option<Timestamp>,
    end: Option<Timestamp>,
    location: Option<String>,
    summary: Option<String>,
}
