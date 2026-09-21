#![cfg(feature = "openapi")]

use serde_json::{Value, json};
use toncenter::v2::endpoints::METHODS;

fn inspect(value: &Value, document: &Value, path: &str, errors: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref").and_then(Value::as_str)
                && (!reference.starts_with('#') || document.pointer(&reference[1..]).is_none())
            {
                errors.push(format!("{path}: unresolved {reference}"));
            }
            if let Some(properties) = object.get("properties").and_then(Value::as_object) {
                for (name, schema) in properties {
                    if schema
                        .get("description")
                        .and_then(Value::as_str)
                        .is_none_or(str::is_empty)
                    {
                        errors.push(format!(
                            "{path}/properties/{name}: missing field description"
                        ));
                    }
                }
            }
            for (key, item) in object {
                inspect(item, document, &format!("{path}/{key}"), errors);
            }
        }
        Value::Array(array) => {
            for (index, item) in array.iter().enumerate() {
                inspect(item, document, &format!("{path}/{index}"), errors);
            }
        }
        _ => {}
    }
}

#[test]
fn all_fields_are_documented_and_all_schema_references_resolve() {
    let document = serde_json::to_value(toncenter::v2::openapi::document()).expect("OpenAPI JSON");
    let mut errors = Vec::new();
    inspect(&document, &document, "", &mut errors);
    for (path, operations) in document["paths"].as_object().expect("paths") {
        for (method, operation) in operations.as_object().expect("operations") {
            for parameter in operation["parameters"].as_array().into_iter().flatten() {
                if parameter["description"].as_str().is_none_or(str::is_empty) {
                    errors.push(format!(
                        "{method} {path}: undocumented query parameter {}",
                        parameter["name"]
                    ));
                }
            }
        }
    }
    expect_test::expect![["[]\n"]].assert_debug_eq(&errors);
}

#[test]
fn checked_in_openapi_matches_rust_types() {
    let generated = toncenter::v2::openapi::document()
        .to_pretty_json()
        .expect("OpenAPI JSON")
        + "\n";
    expect_test::expect_file!["../openapi.json"].assert_eq(&generated);
}

#[test]
fn routes_have_typed_requests_and_distinct_success_schemas() {
    let document = serde_json::to_value(toncenter::v2::openapi::document()).expect("OpenAPI JSON");
    let mut routes = serde_json::Map::new();
    for method in METHODS {
        let path = format!("/api/v2/{method}");
        let operation = &document["paths"][&path];
        routes.insert((*method).to_owned(), json!({
            "get": operation.get("get").is_some(),
            "request": operation["post"]["requestBody"]["content"]["application/json"]["schema"],
            "response": operation["post"]["responses"]["200"]["content"]["application/json"]["schema"],
        }));
    }
    expect_test::expect_file!["fixtures/routes.json"]
        .assert_eq(&(serde_json::to_string_pretty(&routes).expect("routes") + "\n"));
}

#[test]
fn legacy_tuple_schema_rejects_wrong_arity_and_wrong_tags() {
    use utoipa::PartialSchema;
    let mut schema =
        serde_json::to_value(toncenter::v2::stack::LegacyStackEntry::schema()).expect("schema");
    schema["components"] =
        serde_json::to_value(toncenter::v2::openapi::document().components).expect("components");
    let validator = jsonschema::validator_for(&schema).expect("legacy stack schema");
    let results: Vec<bool> = [
        json!([]),
        json!(["num"]),
        json!(["num", "1", false]),
        json!(["wrong", "1"]),
        json!(["cell", 1]),
        json!(["num", "-0x1"]),
    ]
    .iter()
    .map(|value| validator.is_valid(value))
    .collect();
    expect_test::expect![[r"
        [
            false,
            false,
            false,
            false,
            false,
            true,
        ]
    "]]
    .assert_debug_eq(&results);
}
