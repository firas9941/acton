//! Export a v2 or v3 contract, or fail if the checked-in artifact has drifted.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1).peekable();
    let document = if args.peek().is_some_and(|arg| arg == "--v3") {
        args.next();
        toncenter::v3::openapi::document()
    } else {
        toncenter::v2::openapi::document()
    };
    let document = document.to_pretty_json()? + "\n";
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
