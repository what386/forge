use forge::storage::trust::TrustManager;
use std::fs;

fn mk_dir(name: &str) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let trust_file = tmp.path().join("trust").join("trusted.json");
    let dir = tmp.path().join(name);
    fs::create_dir_all(&dir).expect("mkdir");
    (tmp, trust_file, dir)
}

#[test]
fn trusts_and_validates_directory() {
    let (_tmp, trust_file, dir) = mk_dir("template");
    fs::write(dir.join("a.txt"), "hello").expect("write");

    let tm = TrustManager::new(trust_file);
    tm.trust_dir(&dir).expect("trust");
    assert!(tm.is_dir_trusted(&dir).expect("is trusted"));
}

#[test]
fn invalidates_when_contents_change() {
    let (_tmp, trust_file, dir) = mk_dir("template");
    fs::write(dir.join("a.txt"), "hello").expect("write");

    let tm = TrustManager::new(trust_file);
    tm.trust_dir(&dir).expect("trust");
    fs::write(dir.join("a.txt"), "changed").expect("rewrite");

    assert!(!tm.is_dir_trusted(&dir).expect("checked"));
    assert!(tm.list_entries().expect("list").is_empty());
}

#[test]
fn invalidates_when_file_set_changes() {
    let (_tmp, trust_file, dir) = mk_dir("template");
    fs::write(dir.join("a.txt"), "hello").expect("write");

    let tm = TrustManager::new(trust_file);
    tm.trust_dir(&dir).expect("trust");
    fs::write(dir.join("b.txt"), "new").expect("add");

    assert!(!tm.is_dir_trusted(&dir).expect("checked"));
}

#[test]
fn revoke_removes_entry() {
    let (_tmp, trust_file, dir) = mk_dir("template");
    fs::write(dir.join("a.txt"), "hello").expect("write");
    let tm = TrustManager::new(trust_file);
    tm.trust_dir(&dir).expect("trust");

    assert!(tm.revoke_dir(&dir).expect("revoke"));
    assert!(!tm.revoke_dir(&dir).expect("revoke second"));
    assert!(!tm.is_dir_trusted(&dir).expect("trusted"));
}

#[test]
fn missing_store_is_empty() {
    let (_tmp, trust_file, dir) = mk_dir("template");
    fs::write(dir.join("a.txt"), "hello").expect("write");
    let tm = TrustManager::new(trust_file);
    assert!(!tm.is_dir_trusted(&dir).expect("trusted"));
}

#[test]
fn deterministic_checksum_across_rechecks() {
    let (_tmp, trust_file, dir) = mk_dir("template");
    fs::create_dir_all(dir.join("nested")).expect("nested");
    fs::write(dir.join("nested").join("b.txt"), "world").expect("b");
    fs::write(dir.join("a.txt"), "hello").expect("a");

    let tm = TrustManager::new(trust_file);
    tm.trust_dir(&dir).expect("trust");
    assert!(tm.is_dir_trusted(&dir).expect("pass1"));
    assert!(tm.is_dir_trusted(&dir).expect("pass2"));
}

#[test]
fn invalid_json_returns_error() {
    let (_tmp, trust_file, dir) = mk_dir("template");
    fs::write(dir.join("a.txt"), "hello").expect("write");
    fs::create_dir_all(trust_file.parent().expect("parent")).expect("parent mkdir");
    fs::write(&trust_file, "{not valid json").expect("bad json");

    let tm = TrustManager::new(trust_file);
    assert!(tm.list_entries().is_err());
}
