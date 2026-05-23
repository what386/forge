use crate::lua::errors::{ErrorKind, LuaError};
use crate::lua::runtime::{Runtime, RuntimeState};
use crate::templates::manifest::Permission;
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
            let cfg = st.borrow().cfg.clone();
            let p = safe_project_path_with_escape(
                &cfg.project_dir,
                &rel,
                cfg.has_permission(Permission::EscapeCwd),
            )
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
            let cfg = st.borrow().cfg.clone();
            let p = safe_project_path_with_escape(
                &cfg.project_dir,
                &rel,
                cfg.has_permission(Permission::EscapeCwd),
            )
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
            let cfg = st.borrow().cfg.clone();
            let p = safe_project_path_with_escape(
                &cfg.project_dir,
                &rel,
                cfg.has_permission(Permission::EscapeCwd),
            )
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
    let st = state.clone();
    fs_t.set(
        "add",
        lua.create_function(move |lua, (src_rel, dst_rel): (String, String)| {
            Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
            let template_files = st.borrow().cfg.template_dir.join("files");
            let src =
                safe_template_path(&template_files, &src_rel).map_err(mlua::Error::external)?;
            if !src.is_file() {
                return Err(mlua::Error::external(LuaError::new(
                    ErrorKind::SandboxViolation,
                    format!("source file not found: {}", src_rel),
                )));
            }
            let cfg = st.borrow().cfg.clone();
            let dst = safe_project_path_with_escape(
                &cfg.project_dir,
                &dst_rel,
                cfg.has_permission(Permission::EscapeCwd),
            )
            .map_err(mlua::Error::external)?;
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    mlua::Error::external(LuaError::new(ErrorKind::SandboxViolation, e.to_string()))
                })?;
            }
            fs::copy(src, dst).map_err(|e| {
                mlua::Error::external(LuaError::new(ErrorKind::SandboxViolation, e.to_string()))
            })?;
            Ok(())
        })
        .map_err(lua_err)?,
    )
    .map_err(lua_err)?;
    let st = state.clone();
    fs_t.set(
        "remove",
        lua.create_function(move |lua, rel: String| {
            Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
            let cfg = st.borrow().cfg.clone();
            let p = safe_project_path_with_escape(
                &cfg.project_dir,
                &rel,
                cfg.has_permission(Permission::EscapeCwd),
            )
            .map_err(mlua::Error::external)?;
            if p.is_dir() {
                fs::remove_dir_all(p).map_err(|e| {
                    mlua::Error::external(LuaError::new(ErrorKind::SandboxViolation, e.to_string()))
                })?;
            } else if p.exists() {
                fs::remove_file(p).map_err(|e| {
                    mlua::Error::external(LuaError::new(ErrorKind::SandboxViolation, e.to_string()))
                })?;
            }
            Ok(())
        })
        .map_err(lua_err)?,
    )
    .map_err(lua_err)?;
    forge.set("fs", fs_t).map_err(lua_err)
}

pub(crate) fn safe_project_path_with_escape(
    base: &Path,
    rel: &str,
    allow_escape: bool,
) -> Result<PathBuf, LuaError> {
    let rel_path = Path::new(rel);
    if !allow_escape && (rel_path.is_absolute() || has_parent_component(rel_path)) {
        return Err(LuaError::new(
            ErrorKind::SandboxViolation,
            format!("path escapes project dir: {}", rel),
        ));
    }

    let base_abs = base
        .canonicalize()
        .map_err(|e| LuaError::new(ErrorKind::SandboxViolation, e.to_string()))?;
    let joined = if rel_path.is_absolute() {
        rel_path.to_path_buf()
    } else {
        base_abs.join(rel_path)
    };
    let full = resolve_existing_parent(&joined)?;
    if !allow_escape && full != base_abs && !full.starts_with(&base_abs) {
        return Err(LuaError::new(
            ErrorKind::SandboxViolation,
            format!("path escapes project dir: {}", rel),
        ));
    }
    Ok(full)
}

pub(crate) fn safe_template_path(files_root: &Path, rel: &str) -> Result<PathBuf, LuaError> {
    let rel_path = Path::new(rel);
    if rel_path.is_absolute() || has_parent_component(rel_path) {
        return Err(LuaError::new(
            ErrorKind::SandboxViolation,
            format!("path escapes template files dir: {}", rel),
        ));
    }

    let root_abs = files_root
        .canonicalize()
        .map_err(|e| LuaError::new(ErrorKind::SandboxViolation, e.to_string()))?;
    let joined = root_abs.join(rel_path);
    let full = joined.canonicalize().map_err(|_| {
        LuaError::new(
            ErrorKind::SandboxViolation,
            format!("source not found: {}", rel),
        )
    })?;
    if full != root_abs && !full.starts_with(&root_abs) {
        return Err(LuaError::new(
            ErrorKind::SandboxViolation,
            format!("path escapes template files dir: {}", rel),
        ));
    }
    if std::fs::symlink_metadata(&joined)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(LuaError::new(
            ErrorKind::SandboxViolation,
            format!("template symlinks are not allowed: {}", rel),
        ));
    }
    Ok(full)
}

fn resolve_existing_parent(path: &Path) -> Result<PathBuf, LuaError> {
    if path.exists() {
        return path
            .canonicalize()
            .map_err(|e| LuaError::new(ErrorKind::SandboxViolation, e.to_string()));
    }

    let mut existing = path;
    let mut missing = Vec::new();
    while !existing.exists() {
        let Some(parent) = existing.parent() else {
            return Err(LuaError::new(
                ErrorKind::SandboxViolation,
                format!("no existing parent for path: {}", path.display()),
            ));
        };
        if let Some(name) = existing.file_name() {
            missing.push(name.to_os_string());
        }
        existing = parent;
    }

    let mut resolved = existing
        .canonicalize()
        .map_err(|e| LuaError::new(ErrorKind::SandboxViolation, e.to_string()))?;
    for component in missing.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn has_parent_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
}

fn lua_err(err: mlua::Error) -> LuaError {
    LuaError::new(ErrorKind::SandboxViolation, err.to_string())
}
