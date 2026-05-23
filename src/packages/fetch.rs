use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[derive(Debug)]
pub struct FetchedRepo {
    tempdir: TempDir,
}

impl FetchedRepo {
    pub fn path(&self) -> &Path {
        self.tempdir.path()
    }
}

pub fn fetch_repo(url: &str) -> Result<FetchedRepo> {
    let tempdir = tempfile::Builder::new()
        .prefix("forge-package-")
        .tempdir()
        .context("failed to create temporary checkout directory")?;

    let output = Command::new("git")
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg(url)
        .arg(tempdir.path())
        .output()
        .with_context(|| format!("failed to execute git clone for '{}'", url))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        bail!(
            "git clone failed for '{}': {}\n{}{}",
            url,
            output.status,
            stdout,
            stderr
        );
    }

    Ok(FetchedRepo { tempdir })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn run_ok(cmd: &mut Command) {
        let output = cmd.output().expect("command output");
        assert!(
            output.status.success(),
            "command failed: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn fetch_repo_clones_local_repository() {
        let src = tempfile::tempdir().expect("src tempdir");
        run_ok(Command::new("git").arg("init").arg(src.path()));
        run_ok(
            Command::new("git")
                .arg("-C")
                .arg(src.path())
                .arg("config")
                .arg("user.email")
                .arg("test@example.com"),
        );
        run_ok(
            Command::new("git")
                .arg("-C")
                .arg(src.path())
                .arg("config")
                .arg("user.name")
                .arg("Test User"),
        );
        fs::write(src.path().join("README.md"), "hello\n").expect("write fixture");
        run_ok(
            Command::new("git")
                .arg("-C")
                .arg(src.path())
                .arg("add")
                .arg("."),
        );
        run_ok(
            Command::new("git")
                .arg("-C")
                .arg(src.path())
                .arg("commit")
                .arg("-m")
                .arg("init"),
        );

        let fetched = fetch_repo(src.path().to_str().expect("fixture path utf8")).expect("fetch");
        assert!(fetched.path().join("README.md").is_file());
    }

    #[test]
    fn fetch_repo_errors_for_missing_path() {
        let missing = "/tmp/forge-fetch-missing-repo-123456789";
        let err = fetch_repo(missing).expect_err("expected error");
        let msg = format!("{err:#}");
        assert!(msg.contains("git clone failed"));
    }
}
