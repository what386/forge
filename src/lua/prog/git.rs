use crate::lua::errors::{ErrorKind, LuaError};
use crate::lua::exec::{run_exec, ExecMode};
use crate::lua::runtime::{Runtime, RuntimeState};
use crate::lua::types::ExecOptions;
use mlua::{Lua, Table, Variadic};
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) fn register_git(
    lua: &Lua,
    prog: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    let git = lua
        .create_table()
        .map_err(|e| LuaError::new(ErrorKind::Abort, e.to_string()))?;

    let st = state.clone();
    git.set(
        "init",
        lua.create_function(move |lua, ()| {
            Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
            run_exec(
                lua,
                st.clone(),
                vec!["git".to_string(), "init".to_string()],
                ExecOptions::default(),
                ExecMode::Program,
            )
        })
        .map_err(lua_err)?,
    )
    .map_err(lua_err)?;

    let st = state.clone();
    git.set(
        "add",
        lua.create_function(move |lua, args: Variadic<String>| {
            Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
            if args.is_empty() {
                return Err(mlua::Error::external(LuaError::new(
                    ErrorKind::Exec,
                    "forge.prog.git.add requires at least one argument",
                )));
            }
            let mut argv = vec!["git".to_string(), "add".to_string()];
            argv.extend(args);
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
    git.set(
        "commit",
        lua.create_function(move |lua, message: String| {
            Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
            if message.trim().is_empty() {
                return Err(mlua::Error::external(LuaError::new(
                    ErrorKind::Exec,
                    "forge.prog.git.commit requires a non-empty message",
                )));
            }
            run_exec(
                lua,
                st.clone(),
                vec![
                    "git".to_string(),
                    "commit".to_string(),
                    "-m".to_string(),
                    message,
                ],
                ExecOptions::default(),
                ExecMode::Program,
            )
        })
        .map_err(lua_err)?,
    )
    .map_err(lua_err)?;

    prog.set("git", git).map_err(lua_err)
}

fn lua_err(err: mlua::Error) -> LuaError {
    LuaError::new(ErrorKind::Abort, err.to_string())
}
