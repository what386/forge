use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Abort,
    SandboxViolation,
    Render,
    Exec,
}

#[derive(Debug, Clone)]
pub struct LuaError {
    pub kind: ErrorKind,
    pub msg: String,
}

impl LuaError {
    pub fn new(kind: ErrorKind, msg: impl Into<String>) -> Self {
        Self {
            kind,
            msg: msg.into(),
        }
    }
}

impl Display for LuaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for LuaError {}
