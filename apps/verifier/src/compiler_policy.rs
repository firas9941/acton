use thiserror::Error;

/// Parsed compiler restrictions from `compiler.disabled`.
///
/// An empty policy disables nothing. Rules match either every version of a
/// compiler or one exact version, independently of the installed compilers.
#[derive(Clone, Debug, Default)]
pub struct CompilerPolicy {
    disabled: Vec<DisabledCompiler>,
}

impl CompilerPolicy {
    /// Parses compiler names and exact version rules.
    ///
    /// # Errors
    ///
    /// Returns an error identifying the first unknown compiler or malformed version rule.
    pub fn from_disabled(entries: &[String]) -> Result<Self, CompilerPolicyError> {
        let disabled = entries
            .iter()
            .map(|entry| {
                DisabledCompiler::parse(entry).map_err(|reason| CompilerPolicyError {
                    entry: entry.clone(),
                    reason,
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { disabled })
    }

    /// Returns whether any configured rule disables the given compiler version.
    ///
    /// Compiler names are trimmed and matched without ASCII case sensitivity.
    /// Versions are matched verbatim, including prerelease and build suffixes.
    /// This only checks the policy; it does not check compiler availability.
    #[must_use]
    pub fn is_disabled(&self, name: &str, version: &str) -> bool {
        let name = name.trim();
        self.disabled.iter().any(|rule| {
            rule.name.eq_ignore_ascii_case(name)
                && rule.version.as_deref().is_none_or(|exact| exact == version)
        })
    }
}

#[derive(Debug, Error)]
#[error("invalid compiler.disabled entry {entry:?}: {reason}")]
pub struct CompilerPolicyError {
    pub entry: String,
    pub reason: &'static str,
}

#[derive(Clone, Debug)]
struct DisabledCompiler {
    name: &'static str,
    version: Option<String>,
}

impl DisabledCompiler {
    fn parse(entry: &str) -> Result<Self, &'static str> {
        let (name, version) = entry
            .split_once('@')
            .map_or((entry, None), |(name, version)| {
                (name, Some(version.trim()))
            });
        let name = match name.trim().to_ascii_lowercase().as_str() {
            "func" => "func",
            "tolk" => "tolk",
            "tact" => "tact",
            _ => return Err("expected compiler name func, tolk, or tact"),
        };
        if let Some(version) = version {
            semver::Version::parse(version).map_err(|_| {
                "expected an exact version such as 1.4.2; ranges and wildcards are not supported"
            })?;
        }
        Ok(Self {
            name,
            version: version.map(str::to_owned),
        })
    }
}
