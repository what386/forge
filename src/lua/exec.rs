use crate::lua::api::lua_err;
use crate::lua::errors::{ErrorKind, LuaError};
use crate::lua::fs::safe_project_path_with_escape;
use crate::lua::runtime::{Runtime, RuntimeState};
use crate::lua::types::{ExecOptions, ExecResult, ExecRunner};
use crate::templates::manifest::Permission;
use mlua::{Lua, Table};
use std::cell::RefCell;
use std::io::BufRead;
use std::path::Path;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExecMode {
    Raw,
    Program,
}

pub(crate) fn register_exec(
    lua: &Lua,
    forge: &Table,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<(), LuaError> {
    let st = state.clone();
    forge
        .set(
            "exec",
            lua.create_function(move |lua, (cmd_table, opts_t): (Table, Option<Table>)| {
                Runtime::ensure_init(lua, st.clone()).map_err(mlua::Error::external)?;
                let argv = cmd_table
                    .sequence_values::<String>()
                    .collect::<Result<Vec<_>, _>>()?;
                if argv.is_empty() {
                    return Err(mlua::Error::external(LuaError::new(
                        ErrorKind::Exec,
                        "empty command",
                    )));
                }
                let mut opts = ExecOptions::default();
                if let Some(t) = opts_t {
                    opts.cwd = t.get::<Option<String>>("cwd")?.unwrap_or_default();
                    opts.allow_fail = t.get::<Option<bool>>("allow_fail")?.unwrap_or(false);
                    opts.passthrough = t.get::<Option<bool>>("passthrough")?.unwrap_or(false);
                }
                run_exec(lua, st.clone(), argv, opts, ExecMode::Raw)
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)
}

pub(crate) fn run_exec(
    lua: &Lua,
    state: Rc<RefCell<RuntimeState>>,
    argv: Vec<String>,
    opts: ExecOptions,
    mode: ExecMode,
) -> mlua::Result<Table> {
    let cfg = state.borrow().cfg.clone();
    if argv.is_empty() {
        return Err(mlua::Error::external(LuaError::new(
            ErrorKind::Exec,
            "empty command",
        )));
    }
    match mode {
        ExecMode::Raw => {
            if !cfg.has_permission(Permission::Execution) {
                return Err(mlua::Error::external(LuaError::new(
                    ErrorKind::Exec,
                    "forge.exec requires [requires].permissions to include execution",
                )));
            }
            if !command_declared(&argv[0], &cfg.allowed_commands) {
                return Err(mlua::Error::external(LuaError::new(
                    ErrorKind::Exec,
                    format!("command not declared in [requires].commands: {}", argv[0]),
                )));
            }
        }
        ExecMode::Program => {
            if !command_declared(&argv[0], &cfg.allowed_programs) {
                return Err(mlua::Error::external(LuaError::new(
                    ErrorKind::Exec,
                    format!("program not declared in [requires].programs: {}", argv[0]),
                )));
            }
        }
    }

    let cwd = if opts.cwd.is_empty() {
        cfg.project_dir
            .canonicalize()
            .map_err(|e| mlua::Error::external(LuaError::new(ErrorKind::Exec, e.to_string())))?
    } else {
        safe_project_path_with_escape(
            &cfg.project_dir,
            &opts.cwd,
            cfg.has_permission(Permission::EscapeCwd),
        )
        .map_err(mlua::Error::external)?
    };
    let runner: Arc<dyn ExecRunner> = state
        .borrow()
        .cfg
        .exec
        .clone()
        .unwrap_or_else(|| Arc::new(DefaultExecRunner {}));
    let inherit_env = cfg.has_permission(Permission::ReadEnv);
    let env_allowlist = cfg.effective_env_allowlist();
    let res = runner
        .run(&argv, &opts, &cwd, &env_allowlist, inherit_env)
        .map_err(mlua::Error::external)?;
    if !res.ok && !opts.allow_fail {
        return Err(mlua::Error::external(Runtime::abort(
            state.clone(),
            &res.stderr,
        )));
    }
    let out = lua.create_table()?;
    out.set("ok", res.ok)?;
    out.set("code", res.code)?;
    out.set("stdout", res.stdout)?;
    out.set("stderr", res.stderr)?;
    Ok(out)
}

pub struct DefaultExecRunner {}

impl ExecRunner for DefaultExecRunner {
    fn run(
        &self,
        argv: &[String],
        opts: &ExecOptions,
        cwd: &Path,
        env_allowlist: &[String],
        inherit_env: bool,
    ) -> Result<ExecResult, LuaError> {
        let mut cmd = Command::new(&argv[0]);
        if argv.len() > 1 {
            cmd.args(&argv[1..]);
        }
        cmd.current_dir(cwd);
        if !inherit_env {
            cmd.env_clear();
            for key in env_allowlist {
                if let Ok(value) = std::env::var(key) {
                    cmd.env(key, value);
                }
            }
        }
        if opts.passthrough {
            let status = cmd
                .status()
                .map_err(|e| LuaError::new(ErrorKind::Exec, e.to_string()))?;
            let code = status.code().unwrap_or(1);
            return Ok(ExecResult {
                ok: status.success(),
                code,
                ..ExecResult::default()
            });
        }

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd
            .spawn()
            .map_err(|e| LuaError::new(ErrorKind::Exec, e.to_string()))?;
        let mut stdout = String::new();
        let mut stderr = String::new();

        if let Some(out) = child.stdout.take() {
            let reader = std::io::BufReader::new(out);
            for line in reader.lines() {
                let line = line.unwrap_or_default();
                stdout.push_str(&line);
                stdout.push('\n');
            }
        }
        if let Some(err) = child.stderr.take() {
            let reader = std::io::BufReader::new(err);
            for line in reader.lines() {
                let line = line.unwrap_or_default();
                stderr.push_str(&line);
                stderr.push('\n');
            }
        }
        let status = child
            .wait()
            .map_err(|e| LuaError::new(ErrorKind::Exec, e.to_string()))?;
        let code = status.code().unwrap_or(1);
        Ok(ExecResult {
            ok: status.success(),
            code,
            stdout,
            stderr,
        })
    }
}

fn command_declared(argv0: &str, allowed_commands: &[String]) -> bool {
    let Some(file_name) = Path::new(argv0).file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    allowed_commands
        .iter()
        .any(|cmd| cmd == argv0 || cmd == file_name)
}
