use crate::lua::api::register_api;
use crate::lua::errors::{ErrorKind, LuaError};
use crate::lua::sandbox::configure_sandbox;
use crate::lua::types::RuntimeConfig;
use mlua::{Function, Lua, LuaOptions, StdLib, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub(crate) struct RuntimeState {
    pub cfg: RuntimeConfig,
    pub vars: HashMap<String, Value>,
    pub on_init: Option<Function>,
    pub on_complete: Option<Function>,
    pub on_error: Option<Function>,
    pub args_declared: bool,
    pub init_ran: bool,
    pub aborted: bool,
}

pub struct Runtime {
    pub(crate) lua: Lua,
    pub(crate) state: Rc<RefCell<RuntimeState>>,
}

impl Runtime {
    pub fn new(cfg: RuntimeConfig) -> Self {
        let state = RuntimeState {
            cfg,
            vars: HashMap::new(),
            on_init: None,
            on_complete: None,
            on_error: None,
            args_declared: false,
            init_ran: false,
            aborted: false,
        };
        // Scope-aware rendering needs debug.getlocal while render wrappers are installed.
        // The public debug table is removed before any template chunk executes.
        let lua = unsafe {
            Lua::unsafe_new_with(
                (StdLib::ALL_SAFE ^ StdLib::PACKAGE) | StdLib::DEBUG,
                LuaOptions::default(),
            )
        };
        Self {
            lua,
            state: Rc::new(RefCell::new(state)),
        }
    }

    pub fn run(&mut self, main_lua_path: &str) -> Result<(), LuaError> {
        register_api(&self.lua, self.state.clone())?;
        configure_sandbox(&self.lua)?;

        let source = std::fs::read_to_string(main_lua_path)
            .map_err(|e| LuaError::new(ErrorKind::Render, e.to_string()))?;
        if let Err(e) = self.lua.load(&source).set_name(main_lua_path).exec() {
            self.handle_error(&e.to_string());
            return Err(LuaError::new(ErrorKind::Abort, e.to_string()));
        }

        if !self.state.borrow().aborted {
            if let Some(f) = self.state.borrow().on_complete.as_ref() {
                if let Err(e) = f.call::<()>(()) {
                    self.handle_error(&e.to_string());
                    return Err(LuaError::new(ErrorKind::Abort, e.to_string()));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn ensure_init(lua: &Lua, state: Rc<RefCell<RuntimeState>>) -> Result<(), LuaError> {
        if state.borrow().init_ran {
            return Ok(());
        }
        state.borrow_mut().init_ran = true;
        if let Some(init_fn) = state.borrow().on_init.clone() {
            init_fn
                .call::<()>(())
                .map_err(|e| LuaError::new(ErrorKind::Abort, e.to_string()))?;
        }
        let _ = lua;
        Ok(())
    }

    pub(crate) fn abort(state: Rc<RefCell<RuntimeState>>, msg: &str) -> LuaError {
        state.borrow_mut().aborted = true;
        LuaError::new(ErrorKind::Abort, msg)
    }

    fn handle_error(&self, msg: &str) {
        if let Some(f) = self.state.borrow().on_error.as_ref() {
            let _ = f.call::<()>(msg.to_string());
        }
    }
}
