use crate::lua::errors::{ErrorKind, LuaError};
use crate::lua::exec::register_exec;
use crate::lua::fs::register_fs;
use crate::lua::render::register_render;
use crate::lua::runtime::{Runtime, RuntimeState};
use crate::lua::types::{PromptConfirmOptions, PromptInputOptions, PromptSelectOptions};
use crate::templates::manifest::Permission;
use mlua::{Function, Lua, Table, Value};
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) fn register_api(lua: &Lua, state: Rc<RefCell<RuntimeState>>) -> Result<(), LuaError> {
    let forge = lua.create_table().map_err(lua_err)?;
    lua.globals().set("forge", forge.clone()).map_err(lua_err)?;
    register_context(lua, &forge, state.clone())?;
    register_hooks(lua, &forge, state.clone())?;
    register_abort(lua, &forge, state.clone())?;
    register_logging(lua, &forge, state.clone())?;
    register_args(lua, &forge, state.clone())?;
    register_prompt(lua, &forge, state.clone())?;
    register_strings(lua, &forge)?;
    register_env(lua, &forge, state.clone())?;
    register_render(lua, &forge, state.clone())?;
    register_fs(lua, &forge, state.clone())?;
    register_exec(lua, &forge, state.clone())?;
    Ok(())
}

fn register_context(
    lua: &Lua,
    forge: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    let project = lua.create_table().map_err(lua_err)?;
    project
        .set("name", state.borrow().cfg.project_name.clone())
        .map_err(lua_err)?;
    project
        .set(
            "dir",
            state.borrow().cfg.project_dir.to_string_lossy().to_string(),
        )
        .map_err(lua_err)?;
    forge.set("project", project).map_err(lua_err)?;

    let template = lua.create_table().map_err(lua_err)?;
    template
        .set("name", state.borrow().cfg.template_name.clone())
        .map_err(lua_err)?;
    template
        .set(
            "dir",
            state
                .borrow()
                .cfg
                .template_dir
                .to_string_lossy()
                .to_string(),
        )
        .map_err(lua_err)?;
    forge.set("template", template).map_err(lua_err)?;
    forge
        .set("vars", lua.create_table().map_err(lua_err)?)
        .map_err(lua_err)?;
    Ok(())
}

fn register_hooks(
    lua: &Lua,
    forge: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    let st = state.clone();
    forge
        .set(
            "on_init",
            lua.create_function(move |_, f: Function| {
                st.borrow_mut().on_init = Some(f);
                Ok(())
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    let st = state.clone();
    forge
        .set(
            "on_complete",
            lua.create_function(move |_, f: Function| {
                st.borrow_mut().on_complete = Some(f);
                Ok(())
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    let st = state.clone();
    forge
        .set(
            "on_error",
            lua.create_function(move |_, f: Function| {
                st.borrow_mut().on_error = Some(f);
                Ok(())
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    Ok(())
}

fn register_abort(
    lua: &Lua,
    forge: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    forge
        .set(
            "abort",
            lua.create_function(move |_, msg: String| -> mlua::Result<()> {
                Err(mlua::Error::external(Runtime::abort(state.clone(), &msg)))
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)
}

fn register_logging(
    lua: &Lua,
    forge: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    let log = lua.create_table().map_err(lua_err)?;
    for (name, level) in [("info", 0), ("warn", 1), ("error", 2), ("success", 3)] {
        let st = state.clone();
        log.set(
            name,
            lua.create_function(move |_, msg: String| {
                if let Some(logger) = st.borrow().cfg.logger.as_ref() {
                    match level {
                        0 => logger.info(&msg),
                        1 => logger.warn(&msg),
                        2 => logger.error(&msg),
                        _ => logger.success(&msg),
                    }
                }
                Ok(())
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    }
    forge.set("log", log).map_err(lua_err)
}

fn register_args(
    lua: &Lua,
    forge: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    let st = state.clone();
    forge
        .set(
            "args",
            lua.create_function(move |lua, schema: Table| {
                Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
                if st.borrow().args_declared {
                    return Err(mlua::Error::external(LuaError::new(
                        ErrorKind::Abort,
                        "forge.args may only be called once",
                    )));
                }
                st.borrow_mut().args_declared = true;
                let globals = lua.globals();
                let forge_t: Table = globals.get("forge")?;
                let vars: Table = forge_t.get("vars")?;

                for pair in schema.pairs::<String, Table>() {
                    let (name, def) = pair?;
                    let arg_type: String = def
                        .get::<Option<String>>("type")?
                        .unwrap_or_else(|| "string".to_string());
                    let prompt: String = def
                        .get::<Option<String>>("prompt")?
                        .unwrap_or_else(|| name.clone());
                    let default_val: Value = def.get::<Value>("default").unwrap_or(Value::Nil);
                    let val = resolve_arg_value(
                        lua,
                        st.clone(),
                        &name,
                        &arg_type,
                        &prompt,
                        &def,
                        default_val,
                    )?;
                    let validate_fn: Option<Function> = def.get("validate")?;
                    if let Some(vf) = validate_fn {
                        let ret: Value = vf.call(val.clone())?;
                        if !matches!(ret, Value::Nil | Value::Boolean(true)) {
                            return Err(mlua::Error::external(LuaError::new(
                                ErrorKind::Abort,
                                format!("validation failed for \"{}\"", name),
                            )));
                        }
                    }
                    vars.set(name.clone(), val.clone())?;
                    st.borrow_mut().vars.insert(name, val);
                }
                Ok(())
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)
}

fn resolve_arg_value(
    _lua: &Lua,
    state: Rc<RefCell<RuntimeState>>,
    name: &str,
    arg_type: &str,
    prompt: &str,
    def: &Table,
    default_val: Value,
) -> mlua::Result<Value> {
    let prompts = state.borrow().cfg.prompts.clone();
    if prompts.is_none() && !matches!(default_val, Value::Nil) {
        return Ok(default_val);
    }
    let prompts = prompts.ok_or_else(|| {
        mlua::Error::external(LuaError::new(
            ErrorKind::Abort,
            format!("missing prompt provider and no default for \"{}\"", name),
        ))
    })?;
    match arg_type {
        "string" => {
            let out = prompts
                .input(
                    prompt,
                    PromptInputOptions {
                        default: value_to_string(default_val.clone()),
                    },
                )
                .map_err(mlua::Error::external)?;
            if out.is_empty() && !matches!(default_val, Value::Nil) {
                Ok(default_val)
            } else {
                Ok(Value::String(_lua.create_string(out)?))
            }
        }
        "number" => {
            let out = prompts
                .input(
                    prompt,
                    PromptInputOptions {
                        default: value_to_string(default_val.clone()),
                    },
                )
                .map_err(mlua::Error::external)?;
            if out.is_empty() && !matches!(default_val, Value::Nil) {
                Ok(default_val)
            } else {
                let n: f64 = out.parse().map_err(|_| {
                    mlua::Error::external(LuaError::new(
                        ErrorKind::Abort,
                        format!("invalid number for \"{}\"", name),
                    ))
                })?;
                Ok(Value::Number(n))
            }
        }
        "boolean" => {
            let default = matches!(default_val, Value::Boolean(true));
            let out = prompts
                .confirm(prompt, PromptConfirmOptions { default })
                .map_err(mlua::Error::external)?;
            Ok(Value::Boolean(out))
        }
        "select" => {
            let options_t: Option<Table> = def.get("options")?;
            let options_t = options_t.ok_or_else(|| {
                mlua::Error::external(LuaError::new(
                    ErrorKind::Abort,
                    format!("select arg \"{}\" missing options", name),
                ))
            })?;
            let opts = options_t
                .sequence_values::<String>()
                .collect::<Result<Vec<_>, _>>()?;
            if opts.is_empty() {
                return Err(mlua::Error::external(LuaError::new(
                    ErrorKind::Abort,
                    format!("select arg \"{}\" missing options", name),
                )));
            }
            let out = prompts
                .select(PromptSelectOptions {
                    message: prompt.to_string(),
                    options: opts.clone(),
                    default: value_to_string(default_val.clone()),
                })
                .map_err(mlua::Error::external)?;
            if out.is_empty() && !matches!(default_val, Value::Nil) {
                return Ok(default_val);
            }
            if opts.iter().any(|o| o == &out) {
                Ok(Value::String(_lua.create_string(out)?))
            } else {
                Err(mlua::Error::external(LuaError::new(
                    ErrorKind::Abort,
                    format!("invalid option \"{}\" for \"{}\"", out, name),
                )))
            }
        }
        _ => Err(mlua::Error::external(LuaError::new(
            ErrorKind::Abort,
            format!("unsupported arg type \"{}\"", arg_type),
        ))),
    }
}

fn register_prompt(
    lua: &Lua,
    forge: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    let prompt_t = lua.create_table().map_err(lua_err)?;
    let st = state.clone();
    prompt_t
        .set(
            "input",
            lua.create_function(move |lua, (msg, opts): (String, Option<Table>)| {
                Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
                let prompts = st.borrow().cfg.prompts.clone().ok_or_else(|| {
                    mlua::Error::external(LuaError::new(
                        ErrorKind::Abort,
                        "prompt provider not configured",
                    ))
                })?;
                let default = opts
                    .and_then(|t| t.get::<Option<String>>("default").ok().flatten())
                    .unwrap_or_default();
                prompts
                    .input(&msg, PromptInputOptions { default })
                    .map_err(mlua::Error::external)
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    let st = state.clone();
    prompt_t
        .set(
            "confirm",
            lua.create_function(move |lua, (msg, opts): (String, Option<Table>)| {
                Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
                let prompts = st.borrow().cfg.prompts.clone().ok_or_else(|| {
                    mlua::Error::external(LuaError::new(
                        ErrorKind::Abort,
                        "prompt provider not configured",
                    ))
                })?;
                let default = opts
                    .and_then(|t| t.get::<Option<bool>>("default").ok().flatten())
                    .unwrap_or(false);
                prompts
                    .confirm(&msg, PromptConfirmOptions { default })
                    .map_err(mlua::Error::external)
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    let st = state.clone();
    prompt_t
        .set(
            "select",
            lua.create_function(move |lua, schema: Table| {
                Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
                let prompts = st.borrow().cfg.prompts.clone().ok_or_else(|| {
                    mlua::Error::external(LuaError::new(
                        ErrorKind::Abort,
                        "prompt provider not configured",
                    ))
                })?;
                let message = schema
                    .get::<Option<String>>("message")?
                    .unwrap_or_else(|| "Select".to_string());
                let options_t: Table = schema.get("options")?;
                let options = options_t
                    .sequence_values::<String>()
                    .collect::<Result<Vec<_>, _>>()?;
                let default = schema.get::<Option<String>>("default")?.unwrap_or_default();
                prompts
                    .select(PromptSelectOptions {
                        message,
                        options,
                        default,
                    })
                    .map_err(mlua::Error::external)
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    forge.set("prompt", prompt_t).map_err(lua_err)?;
    Ok(())
}

fn register_strings(lua: &Lua, forge: &Table) -> Result<(), LuaError> {
    let str_t = lua.create_table().map_err(lua_err)?;
    str_t
        .set(
            "upper",
            lua.create_function(|_, s: String| Ok(s.to_uppercase()))
                .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    str_t
        .set(
            "lower",
            lua.create_function(|_, s: String| Ok(s.to_lowercase()))
                .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    str_t
        .set(
            "snake",
            lua.create_function(|_, s: String| Ok(to_snake(&s)))
                .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    str_t
        .set(
            "kebab",
            lua.create_function(|_, s: String| Ok(to_kebab(&s)))
                .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    str_t
        .set(
            "pascal",
            lua.create_function(|_, s: String| Ok(to_pascal(&s)))
                .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    str_t
        .set(
            "camel",
            lua.create_function(|_, s: String| Ok(to_camel(&s)))
                .map_err(lua_err)?,
        )
        .map_err(lua_err)?;
    forge.set("str", str_t).map_err(lua_err)?;
    Ok(())
}

fn register_env(
    lua: &Lua,
    forge: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    let env_t = lua.create_table().map_err(lua_err)?;
    let keys = if state.borrow().cfg.has_permission(Permission::ReadEnv) {
        std::env::vars().map(|(key, _)| key).collect()
    } else {
        state.borrow().cfg.effective_env_allowlist()
    };
    for k in keys {
        env_t
            .set(k.clone(), std::env::var(&k).unwrap_or_default())
            .map_err(lua_err)?;
    }
    forge.set("env", env_t).map_err(lua_err)
}

pub(crate) fn value_to_string(v: Value) -> String {
    match v {
        Value::Nil => String::new(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_string_lossy(),
        _ => String::new(),
    }
}

pub(crate) fn lua_err(err: mlua::Error) -> LuaError {
    LuaError::new(ErrorKind::Abort, err.to_string())
}

fn words(input: &str) -> Vec<String> {
    input
        .replace(['-', '_'], " ")
        .split_whitespace()
        .map(|s| s.to_ascii_lowercase())
        .collect()
}

fn to_pascal(input: &str) -> String {
    words(input)
        .into_iter()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn to_camel(input: &str) -> String {
    let pas = to_pascal(input);
    if pas.is_empty() {
        return pas;
    }
    let mut chars = pas.chars();
    let first = chars.next().unwrap().to_ascii_lowercase();
    format!("{}{}", first, chars.as_str())
}
fn to_snake(input: &str) -> String {
    words(input).join("_")
}
fn to_kebab(input: &str) -> String {
    words(input).join("-")
}
