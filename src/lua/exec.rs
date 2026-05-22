use crate::lua::api::lua_err;
use crate::lua::errors::{ErrorKind, LuaError};
use crate::lua::fs::safe_project_path;
use crate::lua::runtime::{Runtime, RuntimeState};
use crate::lua::types::{ExecOptions, ExecResult, ExecRunner};
use mlua::{Lua, Table};
use std::cell::RefCell;
use std::io::BufRead;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::Arc;

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
                if !opts.cwd.is_empty() {
                    safe_project_path(&st.borrow().cfg.project_dir, &opts.cwd)
                        .map_err(mlua::Error::external)?;
                }
                let runner: Arc<dyn ExecRunner> = st
                    .borrow()
                    .cfg
                    .exec
                    .clone()
                    .unwrap_or_else(|| Arc::new(DefaultExecRunner {}));
                let res = runner
                    .run(&argv, &opts, &st.borrow().cfg.project_dir)
                    .map_err(mlua::Error::external)?;
                if !res.ok && !opts.allow_fail {
                    return Err(mlua::Error::external(Runtime::abort(
                        st.clone(),
                        &res.stderr,
                    )));
                }
                let out = lua.create_table()?;
                out.set("ok", res.ok)?;
                out.set("code", res.code)?;
                out.set("stdout", res.stdout)?;
                out.set("stderr", res.stderr)?;
                Ok(out)
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)
}

pub struct DefaultExecRunner {}

impl ExecRunner for DefaultExecRunner {
    fn run(
        &self,
        argv: &[String],
        opts: &ExecOptions,
        project_dir: &std::path::PathBuf,
    ) -> Result<ExecResult, LuaError> {
        let mut cmd = Command::new(&argv[0]);
        if argv.len() > 1 {
            cmd.args(&argv[1..]);
        }
        if opts.cwd.is_empty() {
            cmd.current_dir(project_dir);
        } else {
            cmd.current_dir(project_dir.join(&opts.cwd));
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
