//! `OpenAPI` schemas and routes for the supported v3 operations.

use api::path::{OperationBuilder, ParameterBuilder, ParameterIn, ParameterStyle, PathItemBuilder};
use api::request_body::RequestBodyBuilder;
use api::response::ResponseBuilder;
use api::schema::{Components, Schema};
use api::security::{ApiKey, ApiKeyValue, SecurityRequirement, SecurityScheme};
use api::{Content, Ref, RefOr, Required};
use utoipa::{PartialSchema, ToSchema, openapi as api};

use super::endpoints::{Endpoint, HttpMethod};
use super::responses::RequestError;
use crate::openapi_support::property_description;

/// Builds an `OpenAPI` document for the v3 operations supported by this crate.
/// The document describes direct JSON responses, query parameters, and POST bodies.
#[must_use]
pub fn document() -> api::OpenApi {
    let mut document = Document {
        components: Components::default(),
        paths: api::Paths::default(),
    };
    super::endpoints::register(&mut document);
    document.register::<super::responses::StackValue>();
    document.register::<RequestError>();
    for schema in document.components.schemas.values_mut() {
        if let RefOr::T(Schema::Object(object)) = schema {
            for property in object.properties.values_mut() {
                if let RefOr::T(Schema::OneOf(union)) = property {
                    union.description = union
                        .description
                        .clone()
                        .or_else(|| union.items.iter().find_map(property_description));
                }
            }
        }
    }
    document.components.security_schemes.insert(
        "APIKeyHeader".to_owned(),
        SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
            "X-API-Key",
            "TON Center API key for authenticated access.",
        ))),
    );
    document.components.security_schemes.insert(
        "APIKeyQuery".to_owned(),
        SecurityScheme::ApiKey(ApiKey::Query(ApiKeyValue::with_description(
            "api_key",
            "API key supplied as a query parameter. Prefer the X-API-Key header.",
        ))),
    );

    api::OpenApiBuilder::new()
        .info(api::InfoBuilder::new()
            .title("TON Center API v3 — supported operations")
            .version(super::OPENAPI_VERSION)
            .description(Some("Query indexed accounts, blocks, transactions, traces, tokens, NFTs, and contract state. This document covers the v3 operations supported by the toncenter crate. Native coin amounts use nanograms: 1 GRAM = 1,000,000,000 nanograms. Addresses, hashes, and serialized cells retain their JSON encoding."))
            .license(Some(api::License::new(env!("CARGO_PKG_LICENSE"))))
            .build())
        .servers(Some([api::Server::new("https://toncenter.com"), api::Server::new("https://testnet.toncenter.com")]))
        .paths(document.paths)
        .components(Some(document.components))
        .security(Some([
            SecurityRequirement::new("APIKeyHeader", [] as [&str; 0]),
            SecurityRequirement::new("APIKeyQuery", [] as [&str; 0]),
            SecurityRequirement::default(),
        ]))
        .build()
}

pub(super) struct Document {
    components: Components,
    paths: api::Paths,
}

impl Document {
    pub(super) fn endpoint<E: Endpoint>(&mut self)
    where
        E::Request: ToSchema,
        E::Response: ToSchema,
    {
        self.register::<E::Request>();
        let mut dependencies = Vec::new();
        E::Response::schemas(&mut dependencies);
        self.components.schemas.extend(dependencies);
        let response_schema = E::Response::schema();
        if let RefOr::T(Schema::Object(object)) = &response_schema
            && object.additional_properties.is_none()
        {
            self.components
                .schemas
                .insert(E::Response::name().into_owned(), response_schema.clone());
        }
        let response_name = format!("{}Response", E::OPERATION_ID);
        self.components
            .schemas
            .insert(response_name.clone(), response_schema);

        let mut operation = OperationBuilder::new()
            .operation_id(Some(E::OPERATION_ID))
            .description(Some(E::DESCRIPTION))
            .response("200", ResponseBuilder::new()
                .description("Successful result")
                .content("application/json", Content::new(Some(Ref::from_schema_name(response_name)))))
            .response("default", ResponseBuilder::new()
                .description("API error. Gateways can also return responses without a structured JSON body.")
                .content("application/json", Content::new(Some(Ref::from_schema_name(RequestError::name())))));

        let method = match E::METHOD {
            HttpMethod::Get => {
                if let RefOr::T(Schema::Object(object)) = E::Request::schema() {
                    for (name, schema) in object.properties {
                        let required = object.required.contains(&name);
                        operation = operation.parameter(
                            ParameterBuilder::new()
                                .description(property_description(&schema))
                                .name(name)
                                .parameter_in(ParameterIn::Query)
                                .required(if required {
                                    Required::True
                                } else {
                                    Required::False
                                })
                                .style(Some(ParameterStyle::Form))
                                .explode(Some(true))
                                .schema(Some(schema)),
                        );
                    }
                }
                api::path::HttpMethod::Get
            }
            HttpMethod::Post => {
                operation = operation.request_body(Some(
                    RequestBodyBuilder::new()
                        .description(Some("Parameters encoded as a JSON object."))
                        .required(Some(Required::True))
                        .content(
                            "application/json",
                            Content::new(Some(Ref::from_schema_name(E::Request::name()))),
                        )
                        .build(),
                ));
                api::path::HttpMethod::Post
            }
        };
        self.paths.paths.insert(
            E::PATH.to_owned(),
            PathItemBuilder::new()
                .operation(method, operation.build())
                .build(),
        );
    }

    fn register<T: ToSchema>(&mut self) {
        let mut dependencies = Vec::new();
        T::schemas(&mut dependencies);
        dependencies.push((T::name().into_owned(), T::schema()));
        self.components.schemas.extend(dependencies);
    }
}
