use axum::extract::multipart::Field;

use super::ReceivedFile;
use crate::error::ApiError;

const MAX_UPLOADED_FILES: usize = 256;

pub(super) fn ensure_file_slot(uploaded_file_count: usize) -> Result<(), ApiError> {
    if uploaded_file_count >= MAX_UPLOADED_FILES {
        return Err(ApiError::bad_request(format!(
            "at most {MAX_UPLOADED_FILES} source files may be uploaded"
        )));
    }
    Ok(())
}

pub(super) async fn read_file_part(field: Field<'_>) -> Result<ReceivedFile, ApiError> {
    let file_name = field.file_name().map(ToOwned::to_owned);
    let description = file_name.as_deref().map_or_else(
        || "uploaded file".to_owned(),
        |file_name| format!("uploaded file {file_name}"),
    );
    let content = field.bytes().await.map_err(ApiError::from)?;
    if content.is_empty() {
        return Err(ApiError::bad_request(format!(
            "{description} must not be empty"
        )));
    }

    Ok(ReceivedFile { file_name, content })
}
