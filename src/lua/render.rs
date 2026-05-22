use crate::lua::api::lua_err;
use crate::lua::errors::{ErrorKind, LuaError};
use crate::lua::fs::safe_project_path;
use crate::lua::runtime::{Runtime, RuntimeState};
use mlua::{Function, Lua, Table, Value};
use regex::Regex;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

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

    // The wrapper keeps debug access private after the sandbox removes the public debug table.
    lua.load(
        r#"
        local getlocal = debug.getlocal
        local getinfo = debug.getinfo
        local getupvalue = debug.getupvalue
        local render_native = forge.__render_native
        local render_to_native = forge.__render_to_native

        local function capture_scope(caller_func)
            local scope = {}
            local index = 1
            while true do
                local name, value = getlocal(3, index)
                if name == nil then break end
                if name ~= "(*temporary)" and string.sub(name, 1, 1) ~= "(" then
                    scope[name] = value
                end
                index = index + 1
            end

            if caller_func then
                index = 1
                while true do
                    local name, value = getupvalue(caller_func, index)
                    if name == nil then break end
                    if name ~= "_ENV" and scope[name] == nil then
                        scope[name] = value
                    end
                    index = index + 1
                end
            end
            return scope
        end

        forge.render = function(src)
            local caller = getinfo(2, "f")
            return render_native(src, capture_scope(caller and caller.func))
        end

        forge.render_to = function(src, dst)
            local caller = getinfo(2, "f")
            return render_to_native(src, dst, capture_scope(caller and caller.func))
        end

        forge.__render_native = nil
        forge.__render_to_native = nil
        "#,
    )
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
    let src_abs = cfg.template_dir.join("files").join(src_rel);
    let dst_abs = safe_project_path(&cfg.project_dir, dst_rel)?;
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

fn interpolate(lua: &Lua, input: &str, file: &str, scope: Table) -> Result<String, LuaError> {
    let re = Regex::new(r"\{\{\s*([^}]+)\s*\}\}").expect("valid regex");
    let mut out = String::new();
    let mut last = 0usize;
    for caps in re.captures_iter(input) {
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
