//! Rust-representations of the Events Google Calendar API.
//!
//! See `https://developers.google.com/calendar/api/v3/reference/events`.


use chrono::{DateTime, TimeZone};
use hyper::Body;
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;
use yup_oauth2::AccessToken;

use crate::{calendar::{parse_json_body, request}, impl_builder, impl_get};

use super::GCalTimestamp;

#[derive(Debug)]
pub struct EventsListRequest {
    calendar_id: String,
    max_results: Option<usize>,
    order_by: Option<String>,
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
            order_by: None,
            page_token: None,
            show_deleted: None,
            single_events: None,
            time_max: None,
            time_min: None,
        }
    }

    impl_builder!(
        max_results: Option<usize>,
        order_by: Option<String>,
        page_token: Option<String>,
        show_deleted: Option<bool>,
        single_events: Option<bool>,
    );

    pub fn time_max<T, Tz>(mut self, time: T) -> Self
    where
        T: Into<Option<DateTime<Tz>>>,
        Tz: TimeZone,
        Tz::Offset: fmt::Display,
    {
        self.time_max = time.into().map(|dt| dt.naive_utc().format("%Y-%m-%dT%H:%M:%SZ").to_string());
        self
    }

    pub fn time_min<T, Tz>(mut self, time: T) -> Self
    where
        T: Into<Option<DateTime<Tz>>>,
        Tz: TimeZone,
        Tz::Offset: fmt::Display,
    {
        self.time_min = time.into().map(|dt| dt.naive_utc().format("%Y-%m-%dT%H:%M:%SZ").to_string());
        self
    }

    pub async fn request(self, base_url: &str, token: &AccessToken) -> Option<EventsListResponse> {
        let url = self.to_url(base_url).ok()?;
        parse_json_body(request(token, &url, Body::empty()).await?).await
    }

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

impl EventsListResponse {
    impl_get!(
        items: &[Event]
    );
}

#[derive(Debug, Clone)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    start: GCalTimestamp,
    end: GCalTimestamp,
    location: Option<String>,
    summary: String,
    end_time_unspecified: Option<bool>,
}

impl Event {
    impl_get!(
        start: &GCalTimestamp,
        end: &GCalTimestamp,
        location: &Option<String>,
        summary: &str,
        end_time_unspecified: &Option<bool>,
    );
}
