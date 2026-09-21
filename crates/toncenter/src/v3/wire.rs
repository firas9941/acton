use serde::{Deserialize, Serialize};

/// JSON text or a signed/unsigned 64-bit integer, preserving the received representation.
///
/// Integer JSON values within the signed range decode as `Number`; larger positive
/// values decode as `Unsigned`. Strings are preserved without numeric validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrNumber {
    /// Text supplied by the server, including integers beyond native integer ranges.
    String(String),
    /// JSON integer within the signed 64-bit range.
    Number(i64),
    /// Positive JSON integer that may exceed the signed 64-bit range.
    Unsigned(u64),
}

#[cfg(feature = "openapi")]
impl utoipa::PartialSchema for StringOrNumber {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::schema::AnyOfBuilder::new()
            .description(Some("JSON text or a signed/unsigned 64-bit integer. Strings retain their contents without numeric validation."))
            .item(String::schema())
            .item(i64::schema())
            .item(u64::schema())
            .into()
    }
}

#[cfg(feature = "openapi")]
impl utoipa::ToSchema for StringOrNumber {}

impl std::fmt::Display for StringOrNumber {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value) => formatter.write_str(value),
            Self::Number(value) => value.fmt(formatter),
            Self::Unsigned(value) => value.fmt(formatter),
        }
    }
}

impl From<i32> for StringOrNumber {
    fn from(value: i32) -> Self {
        Self::Number(i64::from(value))
    }
}

impl From<u32> for StringOrNumber {
    fn from(value: u32) -> Self {
        Self::Number(i64::from(value))
    }
}
