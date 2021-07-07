//! Rust-representations of common Google Calendar API types.

use chrono::FixedOffset;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub mod events;

#[macro_export]
macro_rules! impl_builder {
    ( $( $field:ident : $ty:ty ),* $(,)? ) => {
        $(
            #[allow(dead_code)]
            pub fn $field<T: Into<$ty>>(mut self, $field: T) -> Self {
                self.$field = $field.into();
                self
            }
        )*
    }
}

#[macro_export]
macro_rules! impl_get {
    ( $( $field:ident : $ty:ty),* $(,)? ) => {
        $(
            #[allow(dead_code)]
            pub fn $field(&self) -> $ty {
                &self.$field
            }
        )*
    };
}

#[derive(Debug)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GCalTimestamp {
    date: Option<String>,
    date_time: Option<String>,
}

impl GCalTimestamp {
    impl_get!(
        date: &Option<String>,
        date_time: &Option<String>,
    );
}

impl TryFrom<&GCalTimestamp> for Timestamp {
    type Error = ();

    fn try_from(value: &GCalTimestamp) -> Result<Self, Self::Error> {
        if let Some(date_time) = &value.date_time {
            Ok(Timestamp::DateTime(chrono::DateTime::parse_from_rfc3339(&date_time).map_err(|_| ())?))
        } else if let Some(date) = &value.date {
            Ok(Timestamp::Date(chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_err(|_| ())?))
        } else {
            Err(())
        }
    }
}

impl TryFrom<GCalTimestamp> for Timestamp {
    type Error = ();

    fn try_from(value: GCalTimestamp) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

#[derive(Debug)]
pub enum Timestamp {
    Date(chrono::NaiveDate),
    DateTime(chrono::DateTime<FixedOffset>),
}
