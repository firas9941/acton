//! `OpenAPI` generated from the serializable v2 types.

use api::path::{HttpMethod, OperationBuilder, ParameterBuilder, ParameterIn, PathItemBuilder};
use api::request_body::RequestBodyBuilder;
use api::response::ResponseBuilder;
use api::schema::{Components, Schema};
use api::security::{ApiKey, ApiKeyValue, SecurityRequirement, SecurityScheme};
use api::{Content, Ref, RefOr, Required};
use utoipa::{PartialSchema, ToSchema, openapi as api};

use super::endpoints::Endpoint;
use super::{TonlibErrorResponse, TonlibResponse, requests, responses};

/// Builds a complete REST and JSON-RPC contract from Rust types and the endpoint
/// registry. No network access or upstream schema file is needed at runtime.
#[must_use]
pub fn document() -> api::OpenApi {
    let mut document = Document {
        components: Components::default(),
        paths: api::Paths::default(),
    };
    super::endpoints::register(&mut document);
    document.register::<super::stack::LegacyTvmCell>();
    document.register::<super::stack::TvmTuple>();
    document.register::<super::stack::TvmList>();
    document.json_rpc();

    document.components.security_schemes.insert(
        "APIKeyHeader".to_owned(),
        SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
            "X-API-Key",
            "TON Center API key. Keep keys out of logs and shared URLs.",
        ))),
    );
    document.components.security_schemes.insert(
        "APIKeyQuery".to_owned(),
        SecurityScheme::ApiKey(ApiKey::Query(ApiKeyValue::with_description(
            "api_key",
            "API key supplied as a query parameter. Prefer the X-API-Key header.",
        ))),
    );

    for schema in document.components.schemas.values_mut() {
        describe_generated_fields(schema);
    }

    api::OpenApiBuilder::new()
        .info(api::InfoBuilder::new()
            .title("TON Center API v2")
            .version(env!("CARGO_PKG_VERSION"))
            .description(Some("Query TON accounts, blocks, transactions, smart contracts, and configuration through the TON Center v2 REST and JSON-RPC endpoints. Native coin balances use decimal strings in nanograms: 1 GRAM = 1,000,000,000 nanograms. TVM integers use decimal strings in standard stacks and hexadecimal strings in legacy replies. Cell data uses base64-encoded BoCs. Supply an API key in the X-API-Key header for authenticated access."))
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
        self.register::<E::Response>();
        let response_name = format!("{}Response", E::METHOD);
        self.components.schemas.insert(
            response_name.clone(),
            TonlibResponse::<E::Response>::schema(),
        );
        self.register::<TonlibErrorResponse>();

        let success = Content::new(Some(Ref::from_schema_name(response_name)));
        let base = OperationBuilder::new()
            .description(Some(E::DESCRIPTION))
            .response(
                "200",
                ResponseBuilder::new()
                    .description("Successful method result in a TONLib envelope")
                    .content("application/json", success),
            )
            .response(
                "default",
                ResponseBuilder::new()
                    .description(
                        "API or gateway failure, including validation errors and rate limits",
                    )
                    .content(
                        "application/json",
                        Content::new(Some(Ref::from_schema_name(TonlibErrorResponse::name()))),
                    ),
            )
            .build();

        let post = OperationBuilder::from(base.clone())
            .operation_id(Some(format!("{}_post", E::METHOD)))
            .request_body(Some(
                RequestBodyBuilder::new()
                    .description(Some(
                        "Typed parameters. Omit optional fields to use server defaults.",
                    ))
                    .required(Some(Required::True))
                    .content(
                        "application/json",
                        Content::new(Some(Ref::from_schema_name(E::Request::name()))),
                    )
                    .build(),
            ))
            .build();
        let mut path = PathItemBuilder::new().operation(HttpMethod::Post, post);
        if E::SUPPORTS_GET {
            let mut get =
                OperationBuilder::from(base).operation_id(Some(format!("{}_get", E::METHOD)));
            if let RefOr::T(Schema::Object(object)) = E::Request::schema() {
                for (name, schema) in object.properties {
                    let description = property_description(&schema);
                    let required = object.required.contains(&name);
                    get = get.parameter(
                        ParameterBuilder::new()
                            .name(name)
                            .parameter_in(ParameterIn::Query)
                            .description(description)
                            .required(if required {
                                Required::True
                            } else {
                                Required::False
                            })
                            .style(Some(api::path::ParameterStyle::Form))
                            .explode(Some(true))
                            .schema(Some(schema)),
                    );
                }
            }
            path = path.operation(HttpMethod::Get, get.build());
        }
        self.paths.paths.insert(E::PATH.to_owned(), path.build());
    }

    fn register<T: ToSchema>(&mut self) {
        let mut dependencies = Vec::new();
        T::schemas(&mut dependencies);
        dependencies.push((T::name().into_owned(), T::schema()));
        self.components.schemas.extend(dependencies);
    }

    fn json_rpc(&mut self) {
        self.register::<requests::JsonRpcRequest>();
        self.register::<super::Response<responses::RpcResult>>();
        let operation = OperationBuilder::new()
            .operation_id(Some("jsonRPC_post"))
            .description(Some("Invoke any registered v2 method with matching params. The C++ proxy ignores jsonrpc/id metadata and returns the ordinary TONLib envelope, normally without echoing the ID."))
            .request_body(Some(RequestBodyBuilder::new()
                .required(Some(Required::True))
                .content("application/json", Content::new(Some(Ref::from_schema_name(requests::JsonRpcRequest::name()))))
                .build()))
            .response("200", ResponseBuilder::new()
                .description("Method-dependent TONLib result; select the endpoint's response type when decoding")
                .content("application/json", Content::new(Some(Ref::from_schema_name(super::Response::<responses::RpcResult>::name())))))
            .response("default", ResponseBuilder::new()
                .description("Proxy, validation, or TONLib error")
                .content("application/json", Content::new(Some(Ref::from_schema_name(TonlibErrorResponse::name())))))
            .build();
        self.paths.paths.insert(
            "/api/v2/jsonRPC".to_owned(),
            PathItemBuilder::new()
                .operation(HttpMethod::Post, operation)
                .build(),
        );
    }
}

fn property_description(schema: &RefOr<Schema>) -> Option<String> {
    match schema {
        RefOr::T(Schema::Object(value)) => value.description.clone(),
        RefOr::T(Schema::Array(value)) => value.description.clone(),
        RefOr::T(Schema::AllOf(value)) => value.description.clone(),
        RefOr::T(Schema::OneOf(value)) => value
            .description
            .clone()
            .or_else(|| value.items.iter().find_map(property_description)),
        RefOr::T(Schema::AnyOf(value)) => value.description.clone(),
        RefOr::Ref(value) => (!value.description.is_empty()).then(|| value.description.clone()),
        RefOr::T(_) => None,
    }
}

fn describe_generated_fields(value: &mut RefOr<Schema>) {
    let RefOr::T(schema) = value else {
        return;
    };
    match schema {
        Schema::Object(object) => {
            for (name, property) in &mut object.properties {
                if name == "method"
                    && let RefOr::T(Schema::Object(method)) = property
                {
                    method.description = Some(
                        "Case-sensitive v2 method name; determines the type of params and result."
                            .to_owned(),
                    );
                }
                describe_generated_fields(property);
            }
        }
        Schema::Array(array) => {
            if let api::schema::ArrayItems::RefOrSchema(item) = &mut array.items {
                describe_generated_fields(item);
            }
            for item in &mut array.prefix_items {
                let mut wrapped = RefOr::T(item.clone());
                describe_generated_fields(&mut wrapped);
                if let RefOr::T(updated) = wrapped {
                    *item = updated;
                }
            }
        }
        Schema::OneOf(union) => {
            if union.description.is_none() {
                union.description = union.items.iter().find_map(property_description);
            }
            for item in &mut union.items {
                describe_generated_fields(item);
            }
        }
        Schema::AllOf(union) => {
            for item in &mut union.items {
                describe_generated_fields(item);
            }
        }
        Schema::AnyOf(union) => {
            for item in &mut union.items {
                describe_generated_fields(item);
            }
        }
        _ => {}
    }
}
