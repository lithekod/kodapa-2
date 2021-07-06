use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde::de::DeserializeOwned;
use url::Url;
use yup_oauth2::AccessToken;

use crate::calendar::model::events::EventsListResponse;

pub mod model;

const BASE_URL: &'static str = "https://www.googleapis.com/calendar/v3/";
const SCOPES: [&'static str; 1] = [
    "https://www.googleapis.com/auth/calendar",
];

pub async fn handle() {
    let token = token().await;

    let _events = events(&token, "lithekod.se_eos416am56q1g0nuqrtdj8ui1s@group.calendar.google.com").await;
}

pub fn url(endpoint: &str, params: &[(&str, &str)]) -> Url {
    Url::parse_with_params(&format!("{}{}", BASE_URL, endpoint), params).unwrap()
}

async fn token() -> AccessToken {
    let secret = yup_oauth2::read_application_secret("client_secret.json").await.unwrap();
    let authenticator = yup_oauth2::DeviceFlowAuthenticator::builder(secret)
        .persist_tokens_to_disk("tokens.json")
        .build()
        .await
        .unwrap();

    authenticator.token(&SCOPES).await.unwrap()
}

async fn request(token: &AccessToken, url: &Url, body: Body) -> Body {
    let request = Request::builder()
        .uri(url.as_str())
        .header("Authorization", format!("OAuth {}", token.as_str()))
        .body(body)
        .unwrap();

        let https = HttpsConnector::new();
        let client = Client::builder().build(https);
        let response = client.request(request).await.unwrap();
        response.into_body()
}

async fn parse_json_body<T: DeserializeOwned>(body: Body) -> T {
    let bytes = hyper::body::to_bytes(body).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn events(token: &AccessToken, calendar_id: &str) -> Vec<model::events::Event> {
    let url = url(
        &format!("calendars/{}/events", calendar_id),
        &[
            ("maxResults", "6"),
            ("singleEvents", "true"),
            ("timeMin", "2021-07-06T00:00:00Z"),
        ],
    );
    let event_list: EventsListResponse = parse_json_body(request(token, &url, Body::empty()).await).await;
    println!("{:#?}", event_list);
    Vec::new()
}

