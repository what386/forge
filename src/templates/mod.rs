pub mod manifest;
pub mod resolver;
pub mod runner;
pub mod validate;

pub use resolver::{TemplateRecord, TemplateResolver, TemplateSource};
pub use runner::run_template;
pub use validate::validate_template;
