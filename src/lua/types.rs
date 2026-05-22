use crate::lua::errors::LuaError;
use std::path::PathBuf;
use std::sync::Arc;

pub trait Logger: Send + Sync {
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
    fn success(&self, msg: &str);
}

pub trait PromptProvider: Send + Sync {
    fn input(&self, message: &str, opts: PromptInputOptions) -> Result<String, LuaError>;
    fn confirm(&self, message: &str, opts: PromptConfirmOptions) -> Result<bool, LuaError>;
    fn select(&self, opts: PromptSelectOptions) -> Result<String, LuaError>;
}

#[derive(Debug, Clone, Default)]
pub struct PromptInputOptions {
    pub default: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PromptConfirmOptions {
    pub default: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PromptSelectOptions {
    pub message: String,
    pub options: Vec<String>,
    pub default: String,
}

#[derive(Default)]
pub struct ExecOptions {
    pub cwd: String,
    pub allow_fail: bool,
    pub passthrough: bool,
    pub on_stdout: Option<Box<dyn Fn(String) + Send + Sync>>,
    pub on_stderr: Option<Box<dyn Fn(String) + Send + Sync>>,
}

#[derive(Debug, Clone, Default)]
pub struct ExecResult {
    pub ok: bool,
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait ExecRunner: Send + Sync {
    fn run(
        &self,
        argv: &[String],
        opts: &ExecOptions,
        project_dir: &PathBuf,
    ) -> Result<ExecResult, LuaError>;
}

#[derive(Default, Clone)]
pub struct RuntimeConfig {
    pub project_name: String,
    pub project_dir: PathBuf,
    pub template_name: String,
    pub template_dir: PathBuf,
    pub env_allowlist: Vec<String>,
    pub logger: Option<Arc<dyn Logger>>,
    pub prompts: Option<Arc<dyn PromptProvider>>,
    pub exec: Option<Arc<dyn ExecRunner>>,
}
