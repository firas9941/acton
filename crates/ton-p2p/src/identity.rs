use std::{fs, io::Write, path::Path};

use anyhow::{Context, Result};
use rand::RngCore;

/// Reads `adnl.key`, creating a random identity if the file is absent.
///
/// Creation is atomic: concurrent callers reuse whichever key was saved first.
/// The new file is readable and writable only by its owner on Unix.
pub fn load_identity(directory: &Path) -> Result<[u8; 32]> {
    fs::create_dir_all(directory)?;
    let path = directory.join("adnl.key");

    if !path.exists() {
        let mut key = [0_u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
        temporary.write_all(&key)?;
        temporary.as_file().sync_all()?;

        if let Err(error) = temporary.persist_noclobber(&path)
            && error.error.kind() != std::io::ErrorKind::AlreadyExists
        {
            return Err(error.error).context("failed to persist ADNL identity");
        }
    }

    fs::read(&path)
        .with_context(|| format!("failed to read ADNL identity {}", path.display()))?
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid ADNL identity length in {}", path.display()))
}
