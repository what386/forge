use crate::lua::errors::{ErrorKind, LuaError};
use crate::lua::prog::git::register_git;
use crate::lua::runtime::RuntimeState;
use mlua::{Lua, Table};
use std::cell::RefCell;
use std::rc::Rc;

mod git;

pub(crate) fn register_prog(
    lua: &Lua,
    forge: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    let prog = lua
        .create_table()
        .map_err(|e| LuaError::new(ErrorKind::Abort, e.to_string()))?;
    register_git(lua, &prog, state)?;
    forge
        .set("prog", prog)
        .map_err(|e| LuaError::new(ErrorKind::Abort, e.to_string()))
}
