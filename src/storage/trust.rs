use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum TrustError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidPath(String),
}

impl Display for TrustError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{}", e),
            Self::Json(e) => write!(f, "{}", e),
            Self::InvalidPath(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for TrustError {}

impl From<io::Error> for TrustError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for TrustError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

pub type TrustResult<T> = Result<T, TrustError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustedEntry {
    pub path: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TrustStore {
    entries: Vec<TrustedEntry>,
}

pub struct TrustManager {
    store_path: PathBuf,
}

impl TrustManager {
    pub fn new(store_path: PathBuf) -> Self {
        Self { store_path }
    }

    pub fn trust_dir(&self, dir: &Path) -> TrustResult<()> {
        let normalized = normalize_path(dir)?;
        let checksum = checksum_dir(&normalized)?;
        let mut by_path = self.load_entry_map()?;
        by_path.insert(
            normalized.to_string_lossy().to_string(),
            TrustedEntry {
                path: normalized.to_string_lossy().to_string(),
                checksum,
            },
        );
        self.save_entry_map(by_path)
    }

    pub fn is_dir_trusted(&self, dir: &Path) -> TrustResult<bool> {
        let normalized = normalize_path(dir)?;
        let key = normalized.to_string_lossy().to_string();
        let mut by_path = self.load_entry_map()?;
        let Some(existing) = by_path.get(&key).cloned() else {
            return Ok(false);
        };

        let current = checksum_dir(&normalized)?;
        if existing.checksum == current {
            return Ok(true);
        }

        by_path.remove(&key);
        self.save_entry_map(by_path)?;
        Ok(false)
    }

    pub fn revoke_dir(&self, dir: &Path) -> TrustResult<bool> {
        let normalized = normalize_path(dir)?;
        let key = normalized.to_string_lossy().to_string();
        let mut by_path = self.load_entry_map()?;
        let removed = by_path.remove(&key).is_some();
        if removed {
            self.save_entry_map(by_path)?;
        }
        Ok(removed)
    }

    pub fn list_entries(&self) -> TrustResult<Vec<TrustedEntry>> {
        let mut entries: Vec<TrustedEntry> = self.load_entry_map()?.into_values().collect();
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }

    fn load_entry_map(&self) -> TrustResult<BTreeMap<String, TrustedEntry>> {
        if !self.store_path.exists() {
            return Ok(BTreeMap::new());
        }

        let raw = fs::read_to_string(&self.store_path)?;
        let store: TrustStore = serde_json::from_str(&raw)?;
        let mut map = BTreeMap::new();
        for entry in store.entries {
            map.insert(entry.path.clone(), entry);
        }
        Ok(map)
    }

    fn save_entry_map(&self, by_path: BTreeMap<String, TrustedEntry>) -> TrustResult<()> {
        if let Some(parent) = self.store_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut entries: Vec<TrustedEntry> = by_path.into_values().collect();
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        let payload = serde_json::to_vec_pretty(&TrustStore { entries })?;

        let mut tmp = self.store_path.clone();
        tmp.set_extension("tmp");
        fs::write(&tmp, payload)?;
        fs::rename(tmp, &self.store_path)?;
        Ok(())
    }
}

fn normalize_path(path: &Path) -> TrustResult<PathBuf> {
    if let Ok(canonical) = path.canonicalize() {
        return Ok(canonical);
    }

    let base = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir()?
    };
    let absolute = base.join(path);
    Ok(absolute)
}

fn checksum_dir(dir: &Path) -> TrustResult<String> {
    if !dir.exists() {
        return Err(TrustError::InvalidPath(format!(
            "directory does not exist: {}",
            dir.display()
        )));
    }
    if !dir.is_dir() {
        return Err(TrustError::InvalidPath(format!(
            "path is not a directory: {}",
            dir.display()
        )));
    }

    let mut rel_files = Vec::new();
    collect_files(dir, dir, &mut rel_files)?;
    rel_files.sort();

    let mut hasher = Sha256::new();
    for rel in rel_files {
        let full = dir.join(&rel);
        let bytes = fs::read(&full)?;
        hasher.update(rel.as_bytes());
        hasher.update([0u8]);
        hasher.update(&bytes);
        hasher.update([0u8]);
    }

    let digest = hasher.finalize();
    Ok(format!("{:x}", digest))
}

fn collect_files(root: &Path, current: &Path, out: &mut Vec<String>) -> TrustResult<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let path = entry.path();

        if ty.is_symlink() {
            continue;
        }
        if ty.is_dir() {
            collect_files(root, &path, out)?;
            continue;
        }
        if ty.is_file() {
            let rel = path
                .strip_prefix(root)
                .map_err(|e| TrustError::InvalidPath(format!("failed to strip prefix: {}", e)))?;
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}
