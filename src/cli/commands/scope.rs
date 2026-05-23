use anyhow::Result;

use crate::services::paths::PathLayout;
use crate::services::storage::config::{ConfigStorage, TemplateScope};
use crate::templates::ResolveScope;

pub fn resolve(layout: &PathLayout, local: bool, global: bool) -> Result<ResolveScope> {
    if local {
        return Ok(ResolveScope::Local);
    }
    if global {
        return Ok(ResolveScope::Global);
    }

    let config = ConfigStorage::new(&layout.config_file)?;
    Ok(match config.get_config().templates.default_scope {
        TemplateScope::Local => ResolveScope::Local,
        TemplateScope::Global => ResolveScope::Global,
    })
}
