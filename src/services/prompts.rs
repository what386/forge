use crate::lua::{PromptConfirmOptions, PromptInputOptions, PromptProvider, PromptSelectOptions};
use std::io::{self, Write};

pub struct StdioPrompts;
pub struct DefaultPrompts;

impl PromptProvider for StdioPrompts {
    fn input(
        &self,
        message: &str,
        opts: PromptInputOptions,
    ) -> Result<String, crate::lua::LuaError> {
        print!("{}", message);
        if !opts.default.is_empty() {
            print!(" [{}]", opts.default);
        }
        print!(": ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|e| crate::lua::LuaError::new(crate::lua::ErrorKind::Abort, e.to_string()))?;
        let value = line.trim().to_string();
        if value.is_empty() {
            Ok(opts.default)
        } else {
            Ok(value)
        }
    }

    fn confirm(
        &self,
        message: &str,
        opts: PromptConfirmOptions,
    ) -> Result<bool, crate::lua::LuaError> {
        let suffix = if opts.default { "[Y/n]" } else { "[y/N]" };
        print!("{} {}: ", message, suffix);
        let _ = io::stdout().flush();
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|e| crate::lua::LuaError::new(crate::lua::ErrorKind::Abort, e.to_string()))?;
        let value = line.trim().to_ascii_lowercase();
        if value.is_empty() {
            return Ok(opts.default);
        }
        Ok(matches!(value.as_str(), "y" | "yes"))
    }

    fn select(&self, opts: PromptSelectOptions) -> Result<String, crate::lua::LuaError> {
        println!("{}", opts.message);
        for (idx, opt) in opts.options.iter().enumerate() {
            println!("  {}. {}", idx + 1, opt);
        }
        print!("Choice");
        if !opts.default.is_empty() {
            print!(" [{}]", opts.default);
        }
        print!(": ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|e| crate::lua::LuaError::new(crate::lua::ErrorKind::Abort, e.to_string()))?;
        let value = line.trim();
        if value.is_empty() && !opts.default.is_empty() {
            return Ok(opts.default);
        }
        if let Ok(i) = value.parse::<usize>() {
            if i >= 1 && i <= opts.options.len() {
                return Ok(opts.options[i - 1].clone());
            }
        }
        if opts.options.iter().any(|o| o == value) {
            return Ok(value.to_string());
        }
        Err(crate::lua::LuaError::new(
            crate::lua::ErrorKind::Abort,
            "invalid selection",
        ))
    }
}

impl PromptProvider for DefaultPrompts {
    fn input(
        &self,
        _message: &str,
        opts: PromptInputOptions,
    ) -> Result<String, crate::lua::LuaError> {
        Ok(opts.default)
    }

    fn confirm(
        &self,
        _message: &str,
        opts: PromptConfirmOptions,
    ) -> Result<bool, crate::lua::LuaError> {
        Ok(opts.default)
    }

    fn select(&self, opts: PromptSelectOptions) -> Result<String, crate::lua::LuaError> {
        if !opts.default.is_empty() {
            return Ok(opts.default);
        }
        if let Some(first) = opts.options.first() {
            return Ok(first.clone());
        }
        Err(crate::lua::LuaError::new(
            crate::lua::ErrorKind::Abort,
            "select prompt has no default and no options",
        ))
    }
}
