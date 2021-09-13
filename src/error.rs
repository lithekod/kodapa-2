#[derive(Debug)]
pub enum RequestError {
    HttpError(hyper::http::Error),
    HyperError(hyper::Error),
    ResponseError(BodyParseError),
    UrlParseError(url::ParseError),
}

impl From<hyper::http::Error> for RequestError {
    fn from(e: hyper::http::Error) -> Self {
        Self::HttpError(e)
    }
}

impl From<hyper::Error> for RequestError {
    fn from(e: hyper::Error) -> Self {
        Self::HyperError(e)
    }
}

impl From<url::ParseError> for RequestError {
    fn from(e: url::ParseError) -> Self {
        Self::UrlParseError(e)
    }
}

#[derive(Debug)]
pub enum BodyParseError {
    BodyError(hyper::Error),
    JsonError(serde_json::Error),
}

impl From<serde_json::Error> for BodyParseError {
    fn from(e: serde_json::Error) -> Self {
        Self::JsonError(e)
    }
}
