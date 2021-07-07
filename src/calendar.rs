use chrono::{DateTime, Duration, Local, Timelike};
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde::de::DeserializeOwned;
use std::convert::{TryFrom, TryInto};
use url::Url;
use yup_oauth2::AccessToken;

use crate::calendar::model::Timestamp;

use self::model::events::{Event, EventsListRequest, EventsListResponse};

mod model;

const BASE_URL: &'static str = "https://www.googleapis.com/calendar/v3/";
const SCOPES: [&'static str; 1] = [
    "https://www.googleapis.com/auth/calendar",
];

pub async fn handle() {
    let token = token().await.unwrap();
    let calendar_id = "lithekod.se_eos416am56q1g0nuqrtdj8ui1s@group.calendar.google.com".to_string();

    let mut last_fire = None;

    // Send reminder if this is the minute one hour before the event. If the
    // event is at 13.15, we want to send a reminder at 12.15.xx.
    loop {
        println!("sleeping");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        println!("morning");

        let now = Local::now();
        println!("now: {:?}", now);
        println!("last_fire: {:?}", last_fire);
        if last_fire.map(|date| now.date() == date).unwrap_or(false) {
            continue;
        }
        last_fire.take();

        let end = now.checked_add_signed(Duration::minutes(60)).unwrap();
        println!("end: {:?}", end);

        // Try to find a styrelsemöte within 60 minutes.
        let events = events(&token, calendar_id.clone(), now, end).await;
        println!("{} events", events.items().len());
        if let Some(meeting) = events
            .items()
            .iter()
            .find(|event| event.summary() == "Styrelsemöte" && event.start().date_time().is_some())
        {
            println!("found {:?}", meeting);
            // Found a meeting. Notify and mark today as fired.
            let start = match meeting.start().try_into() {
                Ok(Timestamp::DateTime(dt)) => dt,
                _ => panic!("malformed start of event {:?}", meeting),
            };
            last_fire = Some(start.date());
            println!("hello");
        }
    }

}

async fn events(token: &AccessToken, calendar_id: String, start: DateTime<Local>, end: DateTime<Local>) -> EventsListResponse {
    let request = EventsListRequest::new(calendar_id)
        .max_results(50) // Should be enough to not warrant paging
        .order_by("startTime".to_string())
        .single_events(true)
        .time_min(start)
        .time_max(end);

    request.request(BASE_URL, token).await.unwrap()
}

async fn token() -> Option<AccessToken> {
    let secret = yup_oauth2::read_application_secret("client_secret.json").await.ok()?;
    let authenticator = yup_oauth2::DeviceFlowAuthenticator::builder(secret)
        .persist_tokens_to_disk("tokens.json")
        .build()
        .await
        .ok()?;

    authenticator.token(&SCOPES).await.ok()
}

async fn request(token: &AccessToken, url: &Url, body: Body) -> Option<Body> {
    let request = Request::builder()
        .uri(url.as_str())
        .header("Authorization", format!("OAuth {}", token.as_str()))
        .body(body)
        .ok()?;

        let https = HttpsConnector::new();
        let client = Client::builder().build(https);
        let response = client.request(request).await.ok()?;
        Some(response.into_body())
}

async fn parse_json_body<T: DeserializeOwned>(body: Body) -> Option<T> {
    let bytes = hyper::body::to_bytes(body).await.ok()?;
    serde_json::from_slice(&bytes).ok()
}
