//! Rust-representations of the Events Google Calendar API.
//!
//! See `https://developers.google.com/calendar/api/v3/reference/events`.

use serde::{Deserialize, Serialize};
use url::Url;

use super::Timestamp;

macro_rules! impl_builder {
    ( $( $field:ident : $type:ty ),* $(,)? ) => {
        $(
            pub fn $field<T: Into<$type>>(mut self, $field: T) -> Self {
                self.$field = $field.into();
                self
            }
        )*
    }
}

#[derive(Debug)]
pub struct EventsListRequest {
    calendar_id: String,
    max_results: Option<usize>,
    page_token: Option<String>,
    show_deleted: Option<bool>,
    single_events: Option<bool>,
    time_max: Option<String>,
    time_min: Option<String>,
}


impl EventsListRequest {
    pub fn new(calendar_id: String) -> Self {
        Self {
            calendar_id,
            max_results: None,
            page_token: None,
            show_deleted: None,
            single_events: None,
            time_max: None,
            time_min: None,
        }
    }

    impl_builder!(
        max_results: Option<usize>,
        page_token: Option<String>,
        show_deleted: Option<bool>,
        single_events: Option<bool>,
        time_max: Option<String>,
        time_min: Option<String>,
    );

    pub fn to_url(&self, base: &str) -> Result<Url, url::ParseError> {
        let params = self.params();
        let params = if params.is_empty() {
            "".to_string()
        } else {
            format!(
                "?{}",
                params
                    .iter()
                    .map(|(key, val)| format!("{}={}", key, val))
                    .collect::<Vec<_>>()
                    .join("&")
            )
        };
        Url::parse(&format!("{}calendars/{}/events{}", base, self.calendar_id, params))
    }

    pub fn params(&self) -> Vec<(String, String)> {
        macro_rules! push_if_some {
            ($vec:expr, $request:expr, $(($key:expr, $field:ident)),* $(,)?) => {
                $(
                    if let Some(value) = &$request.$field {
                        $vec.push(($key.to_string(), value.to_string()));
                    }
                )*
            };
        }

        let mut res = Vec::new();
        push_if_some!(
            res,
            self,
            ("pageToken", page_token),
            ("showDeleted", show_deleted),
            ("singleEvents", single_events),
            ("timeMax", time_max),
            ("timeMin", time_min),
        );
        res
    }
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
