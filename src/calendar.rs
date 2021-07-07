use chrono::{Duration, Local};
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde::de::DeserializeOwned;
use std::convert::TryFrom;
use url::Url;
use yup_oauth2::AccessToken;

use crate::calendar::model::Timestamp;

use self::model::events::EventsListRequest;

mod model;

const BASE_URL: &'static str = "https://www.googleapis.com/calendar/v3/";
const SCOPES: [&'static str; 1] = [
    "https://www.googleapis.com/auth/calendar",
];

pub async fn handle() {
    let token = token().await;
    let calendar_id = "lithekod.se_eos416am56q1g0nuqrtdj8ui1s@group.calendar.google.com".to_string();

    let now = Local::now();
    let tomorrow = now.checked_add_signed(Duration::days(1)).unwrap();

    let request = EventsListRequest::new(calendar_id)
        .max_results(50) // Should be enough to not warrant paging for now
        .single_events(true)
        .time_min(now)
        .time_max(tomorrow);
    println!("{:?}", request);
    for event in request.request(BASE_URL, &token.unwrap()).await.unwrap().items() {
        println!(
            "{}: {:?}-{:?}",
            event.summary(),
            Timestamp::try_from(event.start()),
            Timestamp::try_from(event.end()),
        );
    }
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
