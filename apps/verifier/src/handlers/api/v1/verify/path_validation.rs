use std::{collections::BTreeMap, path::Component, path::Path};

use crate::error::ApiError;

use super::languages;

const ALLOWED_SOURCE_PATH_PUNCTUATION: [u8; 6] = *b"/._-@+";
const MAX_SOURCE_DIRECTORY_DEPTH: usize = 16;
const MAX_SOURCE_PATH_CHARS: usize = 128;

pub(super) fn validate_source_path(path: &str) -> Result<(), ApiError> {
    if path.chars().count() > MAX_SOURCE_PATH_CHARS {
        return Err(ApiError::bad_request(format!(
            "source path must be no longer than {MAX_SOURCE_PATH_CHARS} characters"
        )));
    }

    validate_relative_path("source path", path)?;
    if path.bytes().filter(|character| *character == b'/').count() > MAX_SOURCE_DIRECTORY_DEPTH {
        return Err(ApiError::bad_request(format!(
            "source path must contain no more than {MAX_SOURCE_DIRECTORY_DEPTH} directories"
        )));
    }
    validate_source_path_components(path)?;
    validate_source_extension_count(path)
}

fn validate_source_path_components(path: &str) -> Result<(), ApiError> {
    if !path.bytes().all(|character| {
        character.is_ascii_alphanumeric() || ALLOWED_SOURCE_PATH_PUNCTUATION.contains(&character)
    }) {
        return Err(ApiError::bad_request(
            "source path components may contain only ASCII letters, numbers, '.', '_', '-', '@' and '+'"
                .to_owned(),
        ));
    }
    if path.split('/').any(|component| component.ends_with('.')) {
        return Err(ApiError::bad_request(
            "source path component must not end with '.'".to_owned(),
        ));
    }

    Ok(())
}

fn validate_source_extension_count(path: &str) -> Result<(), ApiError> {
    let file_name = path
        .rsplit_once('/')
        .map_or(path, |(_, file_name)| file_name);
    let source_extension_count = file_name
        .split('.')
        .skip(1)
        .filter(|extension| languages::is_known_source_extension(extension))
        .take(2)
        .count();
    if source_extension_count > 1 {
        return Err(ApiError::bad_request(
            "source path must not contain multiple source extensions".to_owned(),
        ));
    }

    Ok(())
}

pub(super) fn validate_import_mappings(
    import_mappings: &BTreeMap<String, String>,
) -> Result<(), ApiError> {
    for (prefix, target) in import_mappings {
        validate_relative_path("import mapping prefix", prefix)?;
        validate_relative_path("import mapping target", target)?;
    }

    Ok(())
}

fn validate_relative_path(name: &str, value: &str) -> Result<(), ApiError> {
    if value.trim().is_empty() {
        return Err(ApiError::bad_request(format!("{name} is empty")));
    }
    if value.trim() != value {
        return Err(ApiError::bad_request(format!(
            "{name} has leading or trailing whitespace"
        )));
    }
    if value.chars().any(char::is_control) {
        return Err(ApiError::bad_request(format!(
            "{name} contains a control character"
        )));
    }
    if value.contains('\\') {
        return Err(ApiError::bad_request(format!(
            "{name} must use '/' separators"
        )));
    }
    if value.starts_with('~') {
        return Err(ApiError::bad_request(format!(
            "{name} must not start with '~'"
        )));
    }
    if matches!(
        value.as_bytes(),
        [drive, b':', ..] if drive.is_ascii_alphabetic()
    ) {
        return Err(ApiError::bad_request(format!(
            "{name} must not use a Windows drive prefix"
        )));
    }

    let path = Path::new(value);
    if path.is_absolute() {
        return Err(ApiError::bad_request(format!("{name} must be relative")));
    }

    for component in value.split('/') {
        if component.is_empty() {
            return Err(ApiError::bad_request(format!(
                "{name} contains an empty component"
            )));
        }
        if component == "." {
            return Err(ApiError::bad_request(format!(
                "{name} contains an invalid component"
            )));
        }
        if component.eq_ignore_ascii_case(".git") {
            return Err(ApiError::bad_request(format!(
                "{name} contains reserved '.git' component"
            )));
        }
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return Err(ApiError::bad_request(format!(
                    "{name} contains an invalid component"
                )));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_relative_path, validate_source_path};

    #[test]
    fn source_path_rejects_control_characters() {
        for path in ["main\0.tolk", "main\n.tolk", "main\r.tolk", "main\t.tolk"] {
            assert!(
                validate_source_path(path).is_err(),
                "path should be rejected: {path:?}"
            );
        }
    }

    #[test]
    fn import_mapping_rejects_unsafe_paths() {
        for path in [
            "../contracts",
            "~/contracts",
            "C:/contracts",
            "contracts/./imports",
            "contracts//imports",
            ".git/imports",
            "contracts\n",
        ] {
            assert!(
                validate_relative_path("import mapping target", path).is_err(),
                "import mapping path should be rejected: {path:?}"
            );
        }
    }
}
