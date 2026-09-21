use serde::{Deserialize, Deserializer, Serialize};

macro_rules! integer_input {
    ($name:ident, $integer:ty, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
        #[serde(untagged)]
        pub enum $name {
            /// Decimal text; the server validates syntax and range.
            String(String),
            /// A JSON integer, preserved without floating-point conversion.
            Number($integer),
        }

        impl From<$integer> for $name {
            fn from(value: $integer) -> Self {
                Self::Number(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::String(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::String(value.to_owned())
            }
        }
    };
}

integer_input!(
    Int32Input,
    i32,
    "A request integer accepted as signed 32-bit JSON number or decimal string. The server owns range and semantic validation."
);
integer_input!(
    Int64Input,
    i64,
    "A request integer accepted as signed 64-bit JSON number or decimal string. Use a string when passing through JavaScript to avoid precision loss."
);

/// The archival selector accepts boolean, integer, and string JSON forms.
/// The server validates which textual and numeric values denote true or false.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum BoolInput {
    /// Canonical JSON boolean, recommended for requests produced by Rust.
    Bool(bool),
    /// Numeric boolean accepted by the C++ request parser.
    Number(i32),
    /// Textual boolean accepted by the C++ request parser.
    String(String),
}

impl From<bool> for BoolInput {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

/// Successful REST or JSON-RPC proxy response. Success means the RPC completed;
/// broadcasting success does not prove that a message was included on chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TonlibResponse<T> {
    /// Always true for this envelope; false is rejected during deserialization.
    #[serde(deserialize_with = "deserialize_true")]
    #[cfg_attr(feature = "openapi", schema(schema_with = true_schema))]
    pub ok: bool,
    /// Method-specific result; choose the type associated with the endpoint.
    pub result: T,
    /// Opaque server diagnostic marker. Do not parse it as a stable protocol.
    #[serde(rename = "@extra")]
    pub extra: String,
    /// Optional proxy metadata. The C++ JSON-RPC proxy normally omits this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jsonrpc: Option<String>,
    /// Optional proxy correlation identifier; C++ normally does not echo the request ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Server or gateway failure. HTTP status alone is insufficient to choose the
/// result type: inspect this envelope, including its protocol-specific code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TonlibErrorResponse {
    /// Always false for this envelope; true is rejected during deserialization.
    #[serde(deserialize_with = "deserialize_false")]
    #[cfg_attr(feature = "openapi", schema(schema_with = false_schema))]
    pub ok: bool,
    /// Human-readable diagnostic returned by the server, without a stable grammar.
    pub error: String,
    /// API error code; may differ from an intermediary's HTTP status code.
    pub code: i32,
    /// Opaque diagnostic marker, omitted by some authentication gateways.
    #[serde(default, rename = "@extra", skip_serializing_if = "Option::is_none")]
    pub extra: Option<String>,
    /// Optional proxy version metadata; absent from normal C++ proxy responses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jsonrpc: Option<String>,
    /// Optional proxy correlation identifier; not guaranteed to be echoed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Both possible outcomes of one API call. The `ok` discriminator is validated
/// before a response can be interpreted as success or failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum Response<T> {
    /// The requested method completed and returned its typed result.
    Success(TonlibResponse<T>),
    /// The server rejected or could not execute the request.
    Error(TonlibErrorResponse),
}

impl<T> Response<T> {
    /// Removes the transport envelope while preserving structured API failures.
    /// Applications still own checks such as TVM exit codes and message inclusion.
    pub fn into_result(self) -> Result<T, TonlibErrorResponse> {
        match self {
            Self::Success(response) => Ok(response.result),
            Self::Error(error) => Err(error),
        }
    }
}

impl std::fmt::Display for TonlibErrorResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "TON Center error {}: {}", self.code, self.error)
    }
}

impl std::error::Error for TonlibErrorResponse {}

fn deserialize_true<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    if bool::deserialize(deserializer)? {
        Ok(true)
    } else {
        Err(serde::de::Error::custom(
            "success envelope requires ok=true",
        ))
    }
}

fn deserialize_false<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    if bool::deserialize(deserializer)? {
        Err(serde::de::Error::custom("error envelope requires ok=false"))
    } else {
        Ok(false)
    }
}

#[cfg(feature = "openapi")]
fn true_schema() -> utoipa::openapi::schema::Object {
    utoipa::openapi::schema::ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::Boolean)
        .enum_values(Some([true]))
        .description(Some("Always true for a successful TONLib response"))
        .build()
}

#[cfg(feature = "openapi")]
fn false_schema() -> utoipa::openapi::schema::Object {
    utoipa::openapi::schema::ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::Boolean)
        .enum_values(Some([false]))
        .description(Some("Always false for an API or gateway error"))
        .build()
}
