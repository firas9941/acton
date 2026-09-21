//! Export the v2 contract, or fail if the checked-in artifact has drifted.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let document = toncenter::v2::openapi::document().to_pretty_json()? + "\n";
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--check") => {
            let path = args.next().ok_or("--check requires an OpenAPI file path")?;
            if std::fs::read_to_string(&path)? != document {
                return Err(format!("OpenAPI artifact is stale: {path}").into());
            }
        }
        Some(path) => std::fs::write(path, document)?,
        None => print!("{document}"),
    }
    Ok(())
}
