use crate::lua::errors::{ErrorKind, LuaError};
use crate::lua::exec::{run_exec, ExecMode};
use crate::lua::runtime::{Runtime, RuntimeState};
use crate::lua::types::ExecOptions;
use mlua::{Lua, Table, Variadic};
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) fn register_cargo(
    lua: &Lua,
    prog: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    let cargo = lua
        .create_table()
        .map_err(|e| LuaError::new(ErrorKind::Abort, e.to_string()))?;

    set_passthrough(lua, &cargo, state.clone(), "init")?;
    set_required_arg(lua, &cargo, state.clone(), "new")?;
    set_passthrough(lua, &cargo, state.clone(), "build")?;
    set_passthrough(lua, &cargo, state.clone(), "check")?;
    set_passthrough(lua, &cargo, state.clone(), "test")?;
    set_passthrough(lua, &cargo, state.clone(), "run")?;
    set_passthrough(lua, &cargo, state.clone(), "fmt")?;
    set_passthrough(lua, &cargo, state.clone(), "clippy")?;
    set_alias(
        lua,
        &cargo,
        state.clone(),
        "gen_lockfile",
        "generate-lockfile",
    )?;
    set_required_arg(lua, &cargo, state, "add")?;

    prog.set("cargo", cargo).map_err(lua_err)
}

fn set_passthrough(
    lua: &Lua,
    cargo: &Table,
    state: Rc<RefCell<RuntimeState>>,
    subcommand: &'static str,
) -> Result<(), LuaError> {
    cargo
        .set(
            subcommand,
            lua.create_function(move |lua, args: Variadic<String>| {
                Runtime::ensure_init(lua, state.clone()).map_err(mlua::Error::external)?;
                let mut argv = vec!["cargo".to_string(), subcommand.to_string()];
                argv.extend(args);
                run_exec(
                    lua,
                    state.clone(),
                    argv,
                    ExecOptions::default(),
                    ExecMode::Program,
                )
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)
}

fn set_alias(
    lua: &Lua,
    cargo: &Table,
    state: Rc<RefCell<RuntimeState>>,
    name: &'static str,
    subcommand: &'static str,
) -> Result<(), LuaError> {
    cargo
        .set(
            name,
            lua.create_function(move |lua, args: Variadic<String>| {
                Runtime::ensure_init(lua, state.clone()).map_err(mlua::Error::external)?;
                let mut argv = vec!["cargo".to_string(), subcommand.to_string()];
                argv.extend(args);
                run_exec(
                    lua,
                    state.clone(),
                    argv,
                    ExecOptions::default(),
                    ExecMode::Program,
                )
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)
}

fn set_required_arg(
    lua: &Lua,
    cargo: &Table,
    state: Rc<RefCell<RuntimeState>>,
    subcommand: &'static str,
) -> Result<(), LuaError> {
    cargo
        .set(
            subcommand,
            lua.create_function(move |lua, args: Variadic<String>| {
                Runtime::ensure_init(lua, state.clone()).map_err(mlua::Error::external)?;
                if args.is_empty() || args.iter().all(|arg| arg.trim().is_empty()) {
                    return Err(mlua::Error::external(LuaError::new(
                        ErrorKind::Exec,
                        format!("forge.prog.cargo.{subcommand} requires at least one argument"),
                    )));
                }
                let mut argv = vec!["cargo".to_string(), subcommand.to_string()];
                argv.extend(args);
                run_exec(
                    lua,
                    state.clone(),
                    argv,
                    ExecOptions::default(),
                    ExecMode::Program,
                )
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)
}

fn lua_err(err: mlua::Error) -> LuaError {
    LuaError::new(ErrorKind::Abort, err.to_string())
}
