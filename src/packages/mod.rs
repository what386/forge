pub mod fetch;
pub mod manager;
pub mod probe;

pub use fetch::{fetch_repo, FetchedRepo};
pub use manager::{InstalledTemplate, PackageIndex, PackageManager, PackageRecord};
pub use probe::{probe_repo, ProbedPackage, ProbedTemplate};
