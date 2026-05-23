use forge::templates::manifest::load_and_validate;
use std::fs;

fn write_manifest(dir: &std::path::Path, body: &str) {
    fs::create_dir_all(dir).expect("create dir");
    fs::write(dir.join("manifest.toml"), body).expect("write manifest");
}

#[test]
fn loads_valid_manifest() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let template = tmp.path().join("fullstack");
    write_manifest(
        &template,
        r#"
[package]
name = "fullstack"
version = "1.0.0"
description = "desc"
min_forge_version = "0.1.0"
repository = "https://example.com"

[author]
name = "Alice"

[tags]
values = ["fullstack", "web"]

[requires]
commands = ["cargo"]
programs = ["cargo"]
permissions = ["execution", "network", "read_env"]
"#,
    );

    let manifest = load_and_validate(&template).expect("valid manifest");
    assert_eq!(manifest.package.name, "fullstack");
    assert_eq!(manifest.package.version, "1.0.0");
    assert!(manifest.requires.is_some());
}

#[test]
fn fails_on_missing_manifest() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let err = load_and_validate(tmp.path()).expect_err("should fail");
    assert!(err.to_string().contains("failed to read manifest"));
}

#[test]
fn fails_on_missing_required_field() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let template = tmp.path().join("basic");
    write_manifest(
        &template,
        r#"
[package]
name = "basic"
version = "1.0.0"
min_forge_version = "0.1.0"
"#,
    );

    let err = load_and_validate(&template).expect_err("should fail");
    assert!(err
        .to_string()
        .contains("missing required field: [package].description"));
}

#[test]
fn fails_on_name_dir_mismatch() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let template = tmp.path().join("wrong-dir");
    write_manifest(
        &template,
        r#"
[package]
name = "different"
version = "1.0.0"
description = "desc"
min_forge_version = "0.1.0"
"#,
    );

    let err = load_and_validate(&template).expect_err("should fail");
    assert!(err
        .to_string()
        .contains("must match template directory name"));
}

#[test]
fn fails_on_unknown_top_level_key() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let template = tmp.path().join("basic");
    write_manifest(
        &template,
        r#"
[package]
name = "basic"
version = "1.0.0"
description = "desc"
min_forge_version = "0.1.0"

[unexpected]
foo = "bar"
"#,
    );

    let err = load_and_validate(&template).expect_err("should fail");
    assert!(err.to_string().contains("unknown top-level key"));
}

#[test]
fn fails_on_invalid_permission() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let template = tmp.path().join("basic");
    write_manifest(
        &template,
        r#"
[package]
name = "basic"
version = "1.0.0"
description = "desc"
min_forge_version = "0.1.0"

[requires]
permissions = ["root"]
"#,
    );

    let err = load_and_validate(&template).expect_err("should fail");
    assert!(err
        .to_string()
        .contains("invalid [requires].permissions entry"));
}

#[test]
fn fails_on_missing_required_command() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let template = tmp.path().join("basic");
    write_manifest(
        &template,
        r#"
[package]
name = "basic"
version = "1.0.0"
description = "desc"
min_forge_version = "0.1.0"

[requires]
commands = ["forge_cmd_that_definitely_does_not_exist_123"]
"#,
    );

    let err = load_and_validate(&template).expect_err("should fail");
    assert!(err
        .to_string()
        .contains("required command not found on PATH"));
}

#[test]
fn fails_on_missing_required_program() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let template = tmp.path().join("basic");
    write_manifest(
        &template,
        r#"
[package]
name = "basic"
version = "1.0.0"
description = "desc"
min_forge_version = "0.1.0"

[requires]
programs = ["forge_program_that_definitely_does_not_exist_123"]
"#,
    );

    let err = load_and_validate(&template).expect_err("should fail");
    assert!(err
        .to_string()
        .contains("required program not found on PATH"));
}

#[test]
fn fails_when_min_forge_version_is_higher_than_running() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let template = tmp.path().join("basic");
    write_manifest(
        &template,
        r#"
[package]
name = "basic"
version = "1.0.0"
description = "desc"
min_forge_version = "9999.0.0"
"#,
    );

    let err = load_and_validate(&template).expect_err("should fail");
    assert!(err.to_string().contains("template requires forge 9999.0.0"));
}
