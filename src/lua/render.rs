use crate::lua::api::lua_err;
use crate::lua::errors::{ErrorKind, LuaError};
use crate::lua::fs::{safe_project_path_with_escape, safe_template_path};
use crate::lua::runtime::{Runtime, RuntimeState};
use crate::templates::manifest::Permission;
use mlua::{Function, Lua, Table, Value};
use once_cell::sync::Lazy;
use regex::Regex;
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

static TEMPLATE_BLOCK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{\{\s*([^}]+)\s*\}\}").expect("valid regex"));

pub(crate) fn register_render(
    lua: &Lua,
    forge: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    let st = state.clone();
    forge
        .set(
            "__render_native",
            lua.create_function(move |lua, (src, scope): (String, Table)| {
                Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
                render_file(lua, &st, &src, src.trim_end_matches(".tpl"), scope)
                    .map_err(mlua::Error::external)
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    let st = state.clone();
    forge
        .set(
            "__render_to_native",
            lua.create_function(move |lua, (src, dst, scope): (String, String, Table)| {
                Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
                render_file(lua, &st, &src, &dst, scope).map_err(mlua::Error::external)
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    let st = state.clone();
    forge
        .set(
            "__render_dir_native",
            lua.create_function(move |lua, (src_dir, scope): (String, Table)| {
                Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
                render_dir(lua, &st, &src_dir, &src_dir, scope).map_err(mlua::Error::external)
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    let st = state.clone();
    forge
        .set(
            "__render_dir_to_native",
            lua.create_function(
                move |lua, (src_dir, dst_dir, scope): (String, String, Table)| {
                    Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
                    render_dir(lua, &st, &src_dir, &dst_dir, scope).map_err(mlua::Error::external)
                },
            )
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    // The wrapper keeps debug access private after the sandbox removes the public debug table.
    lua.load(include_str!("scripts/render_wrappers.lua"))
        .set_name("forge.render wrappers")
        .exec()
        .map_err(lua_err)
}

fn render_file(
    lua: &Lua,
    state: &Rc<RefCell<RuntimeState>>,
    src_rel: &str,
    dst_rel: &str,
    scope: Table,
) -> Result<(), LuaError> {
    let cfg = state.borrow().cfg.clone();
    let src_abs = safe_template_path(&cfg.template_dir.join("files"), src_rel)?;
    let dst_abs = safe_project_path_with_escape(
        &cfg.project_dir,
        dst_rel,
        cfg.has_permission(Permission::EscapeCwd),
    )?;
    let mut out = fs::read_to_string(&src_abs)
        .map_err(|e| LuaError::new(ErrorKind::Render, e.to_string()))?;
    if src_rel.ends_with(".tpl") {
        out = interpolate(lua, &out, src_rel, scope)?;
    }
    if let Some(parent) = dst_abs.parent() {
        fs::create_dir_all(parent).map_err(|e| LuaError::new(ErrorKind::Render, e.to_string()))?;
    }
    fs::write(dst_abs, out).map_err(|e| LuaError::new(ErrorKind::Render, e.to_string()))
}

fn render_dir(
    lua: &Lua,
    state: &Rc<RefCell<RuntimeState>>,
    src_dir_rel: &str,
    dst_dir_rel: &str,
    scope: Table,
) -> Result<(), LuaError> {
    let cfg = state.borrow().cfg.clone();
    let src_root = safe_template_path(&cfg.template_dir.join("files"), src_dir_rel)?;
    if !src_root.is_dir() {
        return Err(LuaError::new(
            ErrorKind::Render,
            format!("source directory not found: {}", src_dir_rel),
        ));
    }

    for (abs, rel) in walk_files(&src_root)? {
        let rel_normalized = rel.replace('\\', "/");
        let dst_rel = if dst_dir_rel.is_empty() || dst_dir_rel == "." {
            rel_normalized.clone()
        } else {
            format!("{}/{}", dst_dir_rel.trim_end_matches('/'), rel_normalized)
        };

        if rel_normalized.ends_with(".tpl") {
            let dst_rel = dst_rel.trim_end_matches(".tpl").to_string();
            let src_rel = format!("{}/{}", src_dir_rel.trim_end_matches('/'), rel_normalized);
            let _ = abs;
            render_file(lua, state, &src_rel, &dst_rel, scope.clone())?;
            continue;
        }

        let dst_abs = safe_project_path_with_escape(
            &cfg.project_dir,
            &dst_rel,
            cfg.has_permission(Permission::EscapeCwd),
        )?;
        if let Some(parent) = dst_abs.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| LuaError::new(ErrorKind::Render, e.to_string()))?;
        }
        fs::copy(&abs, dst_abs).map_err(|e| LuaError::new(ErrorKind::Render, e.to_string()))?;
    }

    Ok(())
}

fn walk_files(root: &Path) -> Result<Vec<(std::path::PathBuf, String)>, LuaError> {
    fn inner(
        root: &Path,
        current: &Path,
        out: &mut Vec<(std::path::PathBuf, String)>,
    ) -> Result<(), LuaError> {
        for entry in
            fs::read_dir(current).map_err(|e| LuaError::new(ErrorKind::Render, e.to_string()))?
        {
            let entry = entry.map_err(|e| LuaError::new(ErrorKind::Render, e.to_string()))?;
            let path = entry.path();
            let ty = entry
                .file_type()
                .map_err(|e| LuaError::new(ErrorKind::Render, e.to_string()))?;
            if ty.is_dir() {
                inner(root, &path, out)?;
            } else if ty.is_file() {
                let rel = path
                    .strip_prefix(root)
                    .map_err(|e| LuaError::new(ErrorKind::Render, e.to_string()))?
                    .to_string_lossy()
                    .to_string();
                out.push((path, rel));
            }
        }
        Ok(())
    }

    let mut out = Vec::new();
    inner(root, root, &mut out)?;
    out.sort_by(|a, b| a.1.cmp(&b.1));
    Ok(out)
}

fn interpolate(lua: &Lua, input: &str, file: &str, scope: Table) -> Result<String, LuaError> {
    let mut out = String::new();
    let mut last = 0usize;
    for caps in TEMPLATE_BLOCK_RE.captures_iter(input) {
        let m = caps.get(0).expect("capture");
        out.push_str(&input[last..m.start()]);
        let inner = caps.get(1).map(|x| x.as_str()).unwrap_or("").trim();
        if inner.contains('|') {
            return Err(render_err(
                file,
                inner,
                "pipe helpers are no longer supported; use Lua expressions",
            ));
        }
        let value = eval_expr(lua, &scope, inner).map_err(|e| render_err(file, inner, e))?;
        out.push_str(&value);
        last = m.end();
    }
    out.push_str(&input[last..]);
    Ok(out)
}

fn eval_expr(lua: &Lua, scope: &Table, expr: &str) -> Result<String, String> {
    let env = expression_env(lua, scope).map_err(|e| e.to_string())?;
    let value = lua
        .load(format!("return ({})", expr))
        .set_name("template expression")
        .set_environment(env)
        .eval::<Value>()
        .map_err(|e| e.to_string())?;
    match value {
        Value::Nil => Ok(String::new()),
        value => {
            let tostring: Function = lua.globals().get("tostring").map_err(|e| e.to_string())?;
            tostring.call::<String>(value).map_err(|e| e.to_string())
        }
    }
}

fn expression_env(lua: &Lua, scope: &Table) -> mlua::Result<Table> {
    let env = lua.create_table()?;
    for pair in scope.clone().pairs::<Value, Value>() {
        let (key, value) = pair?;
        env.raw_set(key, value)?;
    }

    let globals = lua.globals();
    let mt = lua.create_table()?;
    mt.set("__index", globals)?;
    env.set_metatable(Some(mt));
    Ok(env)
}

fn render_err(file: &str, expr: &str, message: impl std::fmt::Display) -> LuaError {
    LuaError::new(
        ErrorKind::Render,
        format!(
            "failed to render block {{{{ {} }}}} in {}: {}",
            expr, file, message
        ),
    )
}
