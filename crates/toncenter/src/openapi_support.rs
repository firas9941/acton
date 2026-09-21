use utoipa::openapi::{RefOr, schema::Schema};

pub(crate) fn property_description(schema: &RefOr<Schema>) -> Option<String> {
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
