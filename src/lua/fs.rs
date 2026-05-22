use crate::lua::errors::{ErrorKind, LuaError};
use crate::lua::runtime::{Runtime, RuntimeState};
use mlua::{Lua, Table};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub(crate) fn register_fs(
    lua: &Lua,
    forge: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    let fs_t = lua.create_table().map_err(lua_err)?;
    let st = state.clone();
    fs_t.set(
        "exists",
        lua.create_function(move |lua, rel: String| {
            Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
            let p = safe_project_path(&st.borrow().cfg.project_dir, &rel)
                .map_err(mlua::Error::external)?;
            Ok(p.exists())
        })
        .map_err(lua_err)?,
    )
    .map_err(lua_err)?;

    let st = state.clone();
    fs_t.set(
        "mkdir",
        lua.create_function(move |lua, rel: String| {
            Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
            let p = safe_project_path(&st.borrow().cfg.project_dir, &rel)
                .map_err(mlua::Error::external)?;
            fs::create_dir_all(p).map_err(|e| {
                mlua::Error::external(LuaError::new(ErrorKind::SandboxViolation, e.to_string()))
            })?;
            Ok(())
        })
        .map_err(lua_err)?,
    )
    .map_err(lua_err)?;

    let st = state.clone();
    fs_t.set(
        "write",
        lua.create_function(move |lua, (rel, content): (String, String)| {
            Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
            let p = safe_project_path(&st.borrow().cfg.project_dir, &rel)
                .map_err(mlua::Error::external)?;
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    mlua::Error::external(LuaError::new(ErrorKind::SandboxViolation, e.to_string()))
                })?;
            }
            fs::write(p, content).map_err(|e| {
                mlua::Error::external(LuaError::new(ErrorKind::SandboxViolation, e.to_string()))
            })?;
            Ok(())
        })
        .map_err(lua_err)?,
    )
    .map_err(lua_err)?;
    forge.set("fs", fs_t).map_err(lua_err)
}

pub(crate) fn safe_project_path(base: &Path, rel: &str) -> Result<PathBuf, LuaError> {
    let base_abs = base
        .canonicalize()
        .or_else(|_| std::fs::canonicalize("."))
        .unwrap_or_else(|_| base.to_path_buf());
    let joined = base_abs.join(rel);
    let full = joined
        .canonicalize()
        .unwrap_or_else(|_| joined.components().collect::<PathBuf>());
    if full != base_abs && !full.starts_with(&base_abs) {
        return Err(LuaError::new(
            ErrorKind::SandboxViolation,
            format!("path escapes project dir: {}", rel),
        ));
    }
    Ok(full)
}

fn lua_err(err: mlua::Error) -> LuaError {
    LuaError::new(ErrorKind::SandboxViolation, err.to_string())
}
