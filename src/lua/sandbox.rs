use crate::lua::errors::{ErrorKind, LuaError};
use mlua::{Lua, Value};

pub fn configure_sandbox(lua: &Lua) -> Result<(), LuaError> {
    let globals = lua.globals();
    globals.set("require", Value::Nil).map_err(lua_err)?;
    globals.set("load", Value::Nil).map_err(lua_err)?;
    globals.set("loadfile", Value::Nil).map_err(lua_err)?;
    globals.set("dofile", Value::Nil).map_err(lua_err)?;
    globals.set("debug", Value::Nil).map_err(lua_err)?;
    globals.set("io", Value::Nil).map_err(lua_err)?;
    if let Ok(os_table) = globals.get::<mlua::Table>("os") {
        let _ = os_table.set("execute", Value::Nil);
        let _ = os_table.set("exit", Value::Nil);
    }
    Ok(())
}

fn lua_err(err: mlua::Error) -> LuaError {
    LuaError::new(ErrorKind::SandboxViolation, err.to_string())
}
