use crate::lua::errors::{ErrorKind, LuaError};
use crate::lua::exec::{run_exec, ExecMode};
use crate::lua::runtime::{Runtime, RuntimeState};
use crate::lua::types::ExecOptions;
use mlua::{Lua, Table};
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) fn register_dotnet(
    lua: &Lua,
    prog: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    let dotnet = lua.create_table().map_err(lua_err)?;

    let st = state.clone();
    dotnet
        .set(
            "new",
            lua.create_function(move |lua, (template, options): (String, Option<Table>)| {
                Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
                if template.trim().is_empty() {
                    return Err(mlua::Error::external(LuaError::new(
                        ErrorKind::Exec,
                        "forge.prog.dotnet.new requires a non-empty template",
                    )));
                }

                let mut argv = vec!["dotnet".to_string(), "new".to_string(), template];
                if let Some(options) = options {
                    push_value_option(&options, "name", "--name", &mut argv)?;
                    push_value_option(&options, "output", "--output", &mut argv)?;
                    push_value_option(&options, "format", "--format", &mut argv)?;
                    push_bool_flag(&options, "no_restore", "--no-restore", &mut argv)?;
                }
                run_exec(
                    lua,
                    st.clone(),
                    argv,
                    ExecOptions::default(),
                    ExecMode::Program,
                )
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    let st = state.clone();
    dotnet
        .set(
            "sln_add",
            lua.create_function(move |lua, (solution, project): (String, String)| {
                Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
                require_path("sln_add", "solution", &solution)?;
                require_path("sln_add", "project", &project)?;
                run_dotnet(
                    lua,
                    st.clone(),
                    vec!["sln".to_string(), solution, "add".to_string(), project],
                )
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    let st = state.clone();
    dotnet
        .set(
            "restore",
            lua.create_function(move |lua, (solution, options): (String, Option<Table>)| {
                Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
                require_path("restore", "solution", &solution)?;
                let mut argv = vec!["restore".to_string(), solution];
                if let Some(options) = options {
                    push_bool_flag(&options, "use_lock_file", "--use-lock-file", &mut argv)?;
                }
                run_dotnet(lua, st.clone(), argv)
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    prog.set("dotnet", dotnet).map_err(lua_err)
}

fn run_dotnet(
    lua: &Lua,
    state: Rc<RefCell<RuntimeState>>,
    subcommand: Vec<String>,
) -> mlua::Result<Table> {
    let mut argv = vec!["dotnet".to_string()];
    argv.extend(subcommand);
    run_exec(lua, state, argv, ExecOptions::default(), ExecMode::Program)
}

fn push_value_option(
    options: &Table,
    key: &str,
    flag: &str,
    argv: &mut Vec<String>,
) -> mlua::Result<()> {
    if let Some(value) = options.get::<Option<String>>(key)? {
        if value.trim().is_empty() {
            return Err(mlua::Error::external(LuaError::new(
                ErrorKind::Exec,
                format!("forge.prog.dotnet.new option `{key}` must not be empty"),
            )));
        }
        argv.push(flag.to_string());
        argv.push(value);
    }
    Ok(())
}

fn push_bool_flag(
    options: &Table,
    key: &str,
    flag: &str,
    argv: &mut Vec<String>,
) -> mlua::Result<()> {
    if options.get::<Option<bool>>(key)?.unwrap_or(false) {
        argv.push(flag.to_string());
    }
    Ok(())
}

fn require_path(operation: &str, name: &str, value: &str) -> mlua::Result<()> {
    if value.trim().is_empty() {
        return Err(mlua::Error::external(LuaError::new(
            ErrorKind::Exec,
            format!("forge.prog.dotnet.{operation} requires a non-empty {name}"),
        )));
    }
    Ok(())
}

fn lua_err(err: mlua::Error) -> LuaError {
    LuaError::new(ErrorKind::Abort, err.to_string())
}
