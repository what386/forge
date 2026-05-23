use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub package: Package,
    pub author: Option<Author>,
    pub tags: Option<Tags>,
    pub requires: Option<Requires>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub min_forge_version: String,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Author {
    pub name: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tags {
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requires {
    pub commands: Vec<String>,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    EscapeCwd,
    Network,
    ReadEnv,
}

impl Permission {
    fn from_str(value: &str) -> Result<Self, ManifestError> {
        match value {
            "escape_cwd" => Ok(Self::EscapeCwd),
            "network" => Ok(Self::Network),
            "read_env" => Ok(Self::ReadEnv),
            _ => Err(ManifestError::new(format!(
                "invalid [requires].permissions entry: {}",
                value
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestError {
    msg: String,
}

impl ManifestError {
    fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

impl Display for ManifestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for ManifestError {}

pub fn load_and_validate(template_dir: &Path) -> Result<Manifest, ManifestError> {
    let path = template_dir.join("manifest.toml");
    let raw = fs::read_to_string(&path).map_err(|e| {
        ManifestError::new(format!(
            "failed to read manifest '{}': {}",
            path.display(),
            e
        ))
    })?;

    let root: toml::Value = toml::from_str(&raw)
        .map_err(|e| ManifestError::new(format!("invalid manifest.toml: {}", e)))?;
    let table = root
        .as_table()
        .ok_or_else(|| ManifestError::new("manifest root must be a table"))?;

    validate_top_level_keys(table)?;

    let package_tbl = require_table(table, "package")?;
    let name = require_string(package_tbl, "package", "name")?;
    let version = require_string(package_tbl, "package", "version")?;
    let description = require_string(package_tbl, "package", "description")?;
    let min_forge_version = require_string(package_tbl, "package", "min_forge_version")?;
    let repository = optional_string(package_tbl, "package", "repository")?;

    validate_template_name(&name, template_dir)?;

    let version_semver = parse_semver(&version, "[package].version")?;
    let min_semver = parse_semver(&min_forge_version, "[package].min_forge_version")?;
    let running = parse_semver(env!("CARGO_PKG_VERSION"), "running forge version")?;
    let _ = version_semver;

    if min_semver > running {
        return Err(ManifestError::new(format!(
            "template requires forge {} but running version is {}",
            min_forge_version,
            env!("CARGO_PKG_VERSION")
        )));
    }

    let author = match table.get("author") {
        Some(v) => {
            let t = v
                .as_table()
                .ok_or_else(|| ManifestError::new("[author] must be a table"))?;
            Some(Author {
                name: optional_string(t, "author", "name")?,
                email: optional_string(t, "author", "email")?,
                url: optional_string(t, "author", "url")?,
            })
        }
        None => None,
    };

    let tags = match table.get("tags") {
        Some(v) => {
            let t = v
                .as_table()
                .ok_or_else(|| ManifestError::new("[tags] must be a table"))?;
            let values = optional_string_array(t, "tags", "values")?.unwrap_or_default();
            for tag in &values {
                if !tag
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
                {
                    return Err(ManifestError::new(format!(
                        "[tags].values entries must be lowercase words, got '{}'",
                        tag
                    )));
                }
            }
            Some(Tags { values })
        }
        None => None,
    };

    let requires = match table.get("requires") {
        Some(v) => Some(parse_requires(v)?),
        None => None,
    };

    Ok(Manifest {
        package: Package {
            name,
            version,
            description,
            min_forge_version,
            repository,
        },
        author,
        tags,
        requires,
    })
}

fn parse_requires(value: &toml::Value) -> Result<Requires, ManifestError> {
    let tbl = value
        .as_table()
        .ok_or_else(|| ManifestError::new("[requires] must be a table"))?;

    let commands = optional_string_array(tbl, "requires", "commands")?.unwrap_or_default();
    for cmd in &commands {
        if !command_exists(cmd) {
            return Err(ManifestError::new(format!(
                "required command not found on PATH: {}",
                cmd
            )));
        }
    }

    let permission_strings =
        optional_string_array(tbl, "requires", "permissions")?.unwrap_or_default();
    let mut permissions = Vec::with_capacity(permission_strings.len());
    for p in permission_strings {
        permissions.push(Permission::from_str(&p)?);
    }

    Ok(Requires {
        commands,
        permissions,
    })
}

fn validate_top_level_keys(
    table: &toml::map::Map<String, toml::Value>,
) -> Result<(), ManifestError> {
    let allowed: HashSet<&str> = ["package", "author", "tags", "requires"]
        .into_iter()
        .collect();
    for key in table.keys() {
        if !allowed.contains(key.as_str()) {
            return Err(ManifestError::new(format!(
                "unknown top-level key: [{}]",
                key
            )));
        }
    }
    Ok(())
}

fn validate_template_name(name: &str, template_dir: &Path) -> Result<(), ManifestError> {
    let re = Regex::new(r"^[a-z0-9-]+$").expect("valid regex");
    if !re.is_match(name) {
        return Err(ManifestError::new(
            "[package].name must contain only lowercase letters, numbers, and hyphens",
        ));
    }

    let dir_name = template_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| ManifestError::new("template directory name is not valid UTF-8"))?;

    if name != dir_name {
        return Err(ManifestError::new(format!(
            "[package].name '{}' must match template directory name '{}'",
            name, dir_name
        )));
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Semver {
    major: u64,
    minor: u64,
    patch: u64,
}

fn parse_semver(value: &str, field: &str) -> Result<Semver, ManifestError> {
    let mut parts = value.split('.');
    let major = parts
        .next()
        .ok_or_else(|| ManifestError::new(format!("{} must be MAJOR.MINOR.PATCH", field)))?
        .parse::<u64>()
        .map_err(|_| ManifestError::new(format!("{} must be MAJOR.MINOR.PATCH", field)))?;
    let minor = parts
        .next()
        .ok_or_else(|| ManifestError::new(format!("{} must be MAJOR.MINOR.PATCH", field)))?
        .parse::<u64>()
        .map_err(|_| ManifestError::new(format!("{} must be MAJOR.MINOR.PATCH", field)))?;
    let patch = parts
        .next()
        .ok_or_else(|| ManifestError::new(format!("{} must be MAJOR.MINOR.PATCH", field)))?
        .parse::<u64>()
        .map_err(|_| ManifestError::new(format!("{} must be MAJOR.MINOR.PATCH", field)))?;

    if parts.next().is_some() {
        return Err(ManifestError::new(format!(
            "{} must be MAJOR.MINOR.PATCH",
            field
        )));
    }

    Ok(Semver {
        major,
        minor,
        patch,
    })
}

fn require_table<'a>(
    table: &'a toml::map::Map<String, toml::Value>,
    section: &str,
) -> Result<&'a toml::map::Map<String, toml::Value>, ManifestError> {
    table
        .get(section)
        .and_then(toml::Value::as_table)
        .ok_or_else(|| ManifestError::new(format!("missing required section: [{}]", section)))
}

fn require_string(
    table: &toml::map::Map<String, toml::Value>,
    section: &str,
    key: &str,
) -> Result<String, ManifestError> {
    let value = table.get(key).ok_or_else(|| {
        ManifestError::new(format!("missing required field: [{}].{}", section, key))
    })?;
    match value {
        toml::Value::String(s) if !s.trim().is_empty() => Ok(s.clone()),
        toml::Value::String(_) => Err(ManifestError::new(format!(
            "required field cannot be blank: [{}].{}",
            section, key
        ))),
        _ => Err(ManifestError::new(format!(
            "field must be a string: [{}].{}",
            section, key
        ))),
    }
}

fn optional_string(
    table: &toml::map::Map<String, toml::Value>,
    section: &str,
    key: &str,
) -> Result<Option<String>, ManifestError> {
    let Some(value) = table.get(key) else {
        return Ok(None);
    };
    match value {
        toml::Value::String(s) => Ok(Some(s.clone())),
        _ => Err(ManifestError::new(format!(
            "field must be a string: [{}].{}",
            section, key
        ))),
    }
}

fn optional_string_array(
    table: &toml::map::Map<String, toml::Value>,
    section: &str,
    key: &str,
) -> Result<Option<Vec<String>>, ManifestError> {
    let Some(value) = table.get(key) else {
        return Ok(None);
    };
    let arr = value.as_array().ok_or_else(|| {
        ManifestError::new(format!(
            "field must be an array of strings: [{}].{}",
            section, key
        ))
    })?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let s = item.as_str().ok_or_else(|| {
            ManifestError::new(format!(
                "field must be an array of strings: [{}].{}",
                section, key
            ))
        })?;
        out.push(s.to_string());
    }
    Ok(Some(out))
}

fn command_exists(command: &str) -> bool {
    if command.trim().is_empty() {
        return false;
    }

    let has_sep = command.contains(std::path::MAIN_SEPARATOR);
    if has_sep {
        return PathBuf::from(command).is_file();
    }

    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };

    #[cfg(windows)]
    let exts: Vec<String> = env::var_os("PATHEXT")
        .map(|v| {
            env::split_paths(&PathBuf::from(v))
                .map(|p| p.to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_else(|| vec![".EXE".to_string(), ".BAT".to_string(), ".CMD".to_string()]);

    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return true;
        }
        #[cfg(windows)]
        {
            for ext in &exts {
                let with_ext = dir.join(format!("{}{}", command, ext));
                if with_ext.is_file() {
                    return true;
                }
            }
        }
    }

    false
}
