use std::io::Write;

use verifier::{
    compiler_policy::{CompilerPolicy, CompilerPolicyError},
    config::{Config, ConfigError},
    state::{AppState, StateError},
};

fn load_config(contents: &str) -> Result<Config, ConfigError> {
    let mut file = tempfile::NamedTempFile::new().expect("config file");
    file.write_all(contents.as_bytes()).expect("write config");
    Config::load_from_path(file.path())
}

#[test]
fn absent_and_empty_disabled_lists_disable_nothing() {
    let configs = [
        Config::default(),
        load_config("").expect("empty config"),
        load_config("[compiler]").expect("empty compiler config"),
        load_config("[compiler]\ndisabled = []").expect("empty disabled list"),
        Config::load_from_path("config.toml.example").expect("example config"),
    ];
    for name in ["func", "tolk", "tact"] {
        for version in ["0.4.4", "1.4.1", "99.0.0"] {
            assert!(!CompilerPolicy::default().is_disabled(name, version));
            for config in &configs {
                assert!(config.disabled_compilers().is_empty());
                let policy = CompilerPolicy::from_disabled(config.disabled_compilers())
                    .expect("empty compiler policy");
                assert!(!policy.is_disabled(name, version));
            }
        }
    }
}

#[test]
fn compiler_names_match_all_versions_and_normalize_case_and_whitespace() {
    for name in ["func", "tolk", "tact"] {
        let config = load_config(&format!(
            "[compiler]\ndisabled = [\" {} \"]",
            name.to_ascii_uppercase()
        ))
        .expect("compiler rule");
        let policy =
            CompilerPolicy::from_disabled(config.disabled_compilers()).expect("compiler policy");
        for version in ["0.4.4", "1.4.1", "99.0.0", "0.4.6-wasmfix.0"] {
            assert!(policy.is_disabled(name, version));
            assert!(policy.is_disabled(&format!(" {} ", name.to_ascii_uppercase()), version));
            for other in ["func", "tolk", "tact", "unknown"] {
                assert_eq!(policy.is_disabled(other, version), name == other);
            }
        }
    }
}

#[test]
fn mixed_rules_match_only_the_specified_compilers_and_exact_versions() {
    let config = load_config(
        r#"
[compiler]
disabled = ["tact", "func@0.4.4", " TOLK @ 1.4.1 ", "func@0.4.6-wasmfix.0", "tolk@99.0.0+build.1"]
"#,
    )
    .expect("mixed compiler rules");
    let policy =
        CompilerPolicy::from_disabled(config.disabled_compilers()).expect("compiler policy");

    for (name, version) in [
        ("tact", "1.6.13"),
        ("tact", "99.0.0"),
        ("func", "0.4.4"),
        ("tolk", "1.4.1"),
        (" TOLK ", "1.4.1"),
        ("func", "0.4.6-wasmfix.0"),
        ("tolk", "99.0.0+build.1"),
    ] {
        assert!(policy.is_disabled(name, version), "{name}@{version}");
    }
    for (name, version) in [
        ("func", "0.4.5"),
        ("tolk", "1.4.2"),
        ("func", "1.4.1"),
        ("tolk", "0.4.4"),
        ("func", "0.4.4-newops"),
        ("func", "0.4.6"),
        ("func", "0.4.6-wasmfix.1"),
        ("func", "0.4.6-WASMFIX.0"),
        ("tolk", "99.0.0"),
        ("tolk", "99.0.0+build.2"),
        ("tolk", "v1.4.1"),
        ("tolk", "1.4.1 "),
        ("unknown", "1.4.1"),
    ] {
        assert!(!policy.is_disabled(name, version), "{name}@{version}");
    }
}

#[test]
fn duplicates_and_overlapping_rules_do_not_depend_on_order() {
    for entries in [
        r#"["tact@1.6.13", "tact", "TACT@1.6.13"]"#,
        r#"["tact", "tact@1.6.13", "tact"]"#,
    ] {
        let config = load_config(&format!("[compiler]\ndisabled = {entries}"))
            .expect("overlapping compiler rules");
        let policy =
            CompilerPolicy::from_disabled(config.disabled_compilers()).expect("compiler policy");
        assert!(policy.is_disabled("tact", "1.6.13"));
        assert!(policy.is_disabled("tact", "99.0.0"));
        assert!(!policy.is_disabled("tolk", "1.6.13"));
    }
}

#[test]
fn malformed_rules_are_rejected_by_policy_and_application_initialization() {
    for entry in [
        "",
        " ",
        "unknown",
        "unknown@1.0.0",
        "@1.0.0",
        "tolk@",
        "tolk@ ",
        "tolk@@1.4.1",
        "tolk@1.4.1@1.4.2",
        "tolk@latest",
        "tolk@v1.4.1",
        "tolk@1.4",
        "tolk@01.4.1",
        "tolk@1.4.1-",
        "tolk@1.4.1+",
        "tolk@*",
        "tolk@1.4.*",
        "tolk@^1.4.1",
        "tolk@~1.4.1",
        "tolk@>=1.4.1",
        "tolk@1.4.1, <2.0.0",
        "tolk@1.4.1 - 1.4.2",
    ] {
        let config = load_config(&format!("[compiler]\ndisabled = [\"func\", \"{entry}\"]"))
            .expect("config stores compiler rules as raw strings");
        assert_eq!(config.disabled_compilers(), ["func", entry]);
        let error = CompilerPolicy::from_disabled(config.disabled_compilers()).expect_err(entry);
        let CompilerPolicyError {
            entry: actual,
            reason,
        } = &error;
        assert_eq!(actual, entry);
        assert!(!reason.is_empty());
        assert!(error.to_string().contains("compiler.disabled"));
        assert!(error.to_string().contains(&format!("{entry:?}")));
        assert!(matches!(
            AppState::from_config(&config),
            Err(StateError::CompilerPolicy(_))
        ));
    }
}

#[test]
fn application_initialization_retains_the_parsed_compiler_policy() {
    let directory = tempfile::tempdir().expect("state directory");
    let config = load_config(&format!(
        r#"
[compiler]
disabled = ["tact", "func@0.4.4"]

[registry_index]
path = "{}"

[payment]
address = "0:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
min_amount_nano = 1
ledger_path = "{}"
"#,
        directory.path().join("index.sqlite3").display(),
        directory.path().join("payments.sqlite3").display(),
    ))
    .expect("runtime config");
    let state = AppState::from_config(&config).expect("application state");
    assert!(state.compiler_policy().is_disabled("tact", "1.6.13"));
    assert!(state.compiler_policy().is_disabled("func", "0.4.4"));
    assert!(!state.compiler_policy().is_disabled("func", "0.4.5"));
    assert!(!state.compiler_policy().is_disabled("tolk", "1.4.1"));
}

#[test]
fn disabled_requires_a_list_of_strings() {
    for value in [r#""tact""#, "true", "42", r#"["tact", 42]"#] {
        let error = load_config(&format!("[compiler]\ndisabled = {value}"))
            .expect_err("invalid disabled list");
        assert!(matches!(error, ConfigError::Parse { .. }));
    }
}
