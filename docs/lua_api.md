# Forge Template Engine — v1 Specification

## Overview

Forge is a configurable project scaffolding engine powered by Lua templates.
Templates live locally and are executed through a sandboxed Lua runtime that
exposes controlled access to:

- Project metadata
- File rendering
- Filesystem operations
- User prompts
- Command execution
- Logging

Design goals: deterministic generation, safe execution, reusable templates,
lightweight scripting, fast local workflows.

---

# CLI

## Create a Project

```bash
forge new <template> <name>
```

Example:

```bash
forge new webapp my-app
```

Forge will:

1. Locate the template at `.forge/templates/webapp/`
2. Validate `manifest.toml` (including `min_forge_version`)
3. Execute `.forge/templates/webapp/main.lua`
4. Expose project metadata and APIs to the template runtime
5. Generate output into `./my-app/`

---

# Template Structure

```text
.forge/templates/webapp/
├── main.lua
├── manifest.toml
└── files/
    ├── package.json.tpl
    ├── README.md.tpl
    └── src/
```

---

# Template Metadata

Every template must include a `manifest.toml`:

```toml
name              = "webapp"
version           = "1.0.0"
description       = "Creates a web application project"
min_forge_version = "1.0.0"
```

`min_forge_version` is required. Forge will refuse to execute a template that
requires a newer version and will print a clear error before exiting.

---

# Runtime Environment

Forge exposes a single global table:

```lua
forge
```

No other globals are required. Standard Lua globals that allow unrestricted
system access (`os.execute`, `io.open`, `require`, `load`, `dofile`, `debug.*`)
are removed from the sandbox before template execution begins.

---

# Runtime Context

## Project

```lua
forge.project.name   -- string: the name passed to `forge new`
forge.project.dir    -- string: absolute path to the output directory
```

## Template

```lua
forge.template.name  -- string: template identifier
forge.template.dir   -- string: absolute path to the template directory
```

---

# Arguments

## Declaring Arguments

Call `forge.args(schema)` once, near the top of `main.lua`. Forge collects all
values (via prompts or defaults) before any rendering begins.

```lua
forge.args({
    language = {
        prompt  = "Language",
        type    = "select",
        options = { "typescript", "javascript" },
        default = "typescript"
    },
    port = {
        prompt   = "Dev server port",
        type     = "number",
        default  = 3000,
        validate = function(v)
            return (v > 1024 and v < 65536) or "Must be between 1025 and 65535"
        end
    },
    git = {
        prompt  = "Initialize git repo?",
        type    = "boolean",
        default = true
    },
    author = {
        prompt  = "Author name",
        -- type defaults to "string" when omitted
        default = forge.env.USER
    }
})
```

### Supported types

| Type      | Prompt widget   | Lua value  |
|-----------|-----------------|------------|
| `string`  | Text input      | `string`   |
| `number`  | Text input      | `number`   |
| `boolean` | Confirm (y/n)   | `boolean`  |
| `select`  | Selection list  | `string`   |

`type` defaults to `"string"` when omitted.

`options` is required when `type = "select"`.

`validate` is an optional function. Return `true` (or nothing) to pass;
return a non-empty string to show that message and re-prompt.

## Accessing Variables

```lua
forge.vars.language   -- "typescript"
forge.vars.port       -- 3000
forge.vars.git        -- true
forge.vars.author     -- "alice"
```

---

# Abort

```lua
forge.abort("Directory is not empty")
```

Terminates execution immediately with a clean, user-facing error message.
Does not surface a Lua stack trace. Use this instead of `error()` anywhere
you want to exit with a message the user should actually read.

If an `on_error` hook is registered it runs before the process exits.

---

# Rendering

## Render a File

```lua
forge.render("whatever/file.txt.tpl")
```

Renders a template and writes it to the same relative path in the output
directory, stripping the `.tpl` extension. Source is relative to `files/`.

Use this for the common case where the destination mirrors the source layout.

## Render a File to a Specific Destination

```lua
forge.render_to("backend/go/go.mod.tpl", "backend/go.mod")
```

Renders a template to an explicit destination path. Useful when the output
path differs from the source — e.g. selecting a file based on a variable.

## Render a Directory

```lua
forge.render_dir("frontend/react")
```

Source path is relative to the template's `files/` directory.

- `.tpl` files are rendered and written without the `.tpl` extension
- All other files are copied as-is
- Directory structure is preserved

## Render a Directory to a Specific Destination

```lua
forge.render_dir_to("frontend/react", "client")
```

Renders a directory to an explicit destination path in the output directory.

## Template Interpolation (`.tpl` files)

```
{{ forge.project.name }}                   -- project metadata
{{ language }}                             -- a local visible at forge.render(...)
{{ frontend .. "-" .. backend }}          -- string concat
{{ port + 1 }}                             -- math
{{ forge.str.pascal(forge.project.name) }} -- string helper call
```

Each `{{ ... }}` block is a single Lua expression. Forge evaluates it with the
locals and upvalues visible where `forge.render` or `forge.render_to` is called,
plus the sandboxed globals. A `nil` result renders as an empty string; every
other result is converted with Lua's `tostring`.

Use `%{{ ... }}%` when you need literal template delimiters in the output.
This form is not evaluated; it emits `{{ ... }}` exactly.

```
%{{args}}%      -- renders as: {{args}}
%{{version}}%   -- renders as: {{version}}
```

Use `forge.str.pascal`, `forge.str.camel`, `forge.str.snake`,
`forge.str.kebab`, `forge.str.upper`, and `forge.str.lower` for string
helpers. Pipe helper syntax is not supported.

Conditionals and loops are handled in `main.lua`, not in `.tpl` files. A later
template-block design may add controlled block syntax without allowing arbitrary
Lua chunks inside rendered files.

---

# Filesystem API

All filesystem access is sandboxed through `forge.fs`. Paths are relative to
the project output directory.

```lua
forge.fs.exists(path)
forge.fs.mkdir(path)
forge.fs.write(path, content)
forge.fs.add(src, dst)         -- copy a binary asset from files/ into the output directory
forge.fs.remove(path)          -- primarily useful in on_error cleanup
```

---

# Command Execution

Always pass commands as a table of arguments. Shell string execution is not
supported — no shell interpolation, no injection surface.

```lua
local result = forge.exec({ "git", "init" })
```

## Return Value

```lua
result.ok      -- boolean: true if exit code is 0
result.code    -- number:  raw exit code
result.stdout  -- string:  captured standard output
result.stderr  -- string:  captured standard error
```

## Options

```lua
forge.exec({ "npm", "install" }, {
    cwd         = "frontend",  -- relative to project output dir; default is output dir root
    allow_fail  = false,       -- if false (default), non-zero exit calls forge.abort automatically
    passthrough = false,       -- if true, output is forwarded directly to the terminal
    on_stdout   = function(line) end,  -- called for each stdout line
    on_stderr   = function(line) end,  -- called for each stderr line
})
```

`allow_fail = true` lets the template inspect the result and decide what to do:

```lua
-- Example: detect available package manager
local bun = forge.exec({ "which", "bun" }, { allow_fail = true })
local pm  = bun.ok and "bun" or "npm"
forge.exec({ pm, "install" })
```

`passthrough = true` forwards output directly to the terminal. Use this for
long-running commands where you want the user to see live progress. `result.stdout`
and `result.stderr` will be empty strings since output was not captured.

```lua
-- Example: stream build output to the terminal
forge.exec({ "cargo", "build" }, { passthrough = true })
```

`on_stdout` / `on_stderr` are called line-by-line as output arrives. Use these
when you need to react to output programmatically. Mutually exclusive with
`passthrough`.

```lua
-- Example: parse output line by line
forge.exec({ "npm", "install" }, {
    on_stderr = function(line)
        if line:match("ERR!") then
            forge.log.warn("npm: " .. line)
        end
    end
})
```

---

# Logging

```lua
forge.log.info("Creating project structure")
forge.log.warn("git not found — skipping init")
forge.log.error("Config is invalid")
forge.log.success("Project ready")
```

All levels produce colored, consistently formatted output to stderr.
`forge.log.error` does **not** abort — use `forge.abort()` for that.

---

# User Prompt API

For input that can't be declared statically in `forge.vars` — e.g. prompts
that depend on earlier answers.

```lua
-- Free text
local org = forge.prompt.input("GitHub org", { default = "acme" })

-- Confirm
local yes = forge.prompt.confirm("Enable Docker?", { default = false })

-- Selection
local db = forge.prompt.select({
    message = "Database",
    options = { "postgres", "sqlite", "mysql" },
    default = "postgres"
})

-- Input with validation
local tag = forge.prompt.input("Image tag", {
    validate = function(v)
        return v:match("^[a-z0-9%-]+$") or "Only lowercase letters, numbers, hyphens"
    end
})
```

---

# String Utilities

All helpers accept camelCase, PascalCase, snake_case, or kebab-case as input.

```lua
forge.str.camel("hello-world")   --> "helloWorld"
forge.str.pascal("hello-world")  --> "HelloWorld"
forge.str.snake("hello-world")   --> "hello_world"
forge.str.kebab("helloWorld")    --> "hello-world"
forge.str.upper("hello")         --> "HELLO"
forge.str.lower("HELLO")         --> "hello"
```

---

# Environment Variables

Only an explicit allowlist is exposed. Full environment access is not available.

```lua
forge.env.HOME
forge.env.USER
forge.env.PATH
forge.env.SHELL
```

---

# Hooks

```lua
-- Runs before vars are collected or files are rendered
forge.on_init(function()
    forge.log.info("Starting setup")
end)

-- Runs after all rendering and exec calls complete successfully
forge.on_complete(function()
    forge.log.success("Done! cd " .. forge.project.name .. " && npm run dev")
end)

-- Runs on forge.abort() or any unhandled error; use for cleanup
forge.on_error(function(err)
    forge.log.warn("Error: " .. err)
    forge.fs.remove(forge.project.dir)   -- remove partial output
end)
```

Registering a hook twice replaces the previous handler. Only one handler per
hook is supported.

---

# Security Model

Templates execute inside a sandboxed Lua runtime. The following are removed
from the environment before execution:

| Removed global | Reason |
|---|---|
| `os.execute`, `os.exit` | Arbitrary command execution |
| `io.*` | Raw filesystem access |
| `require` | Loading external Lua modules |
| `load`, `loadfile`, `dofile` | Dynamic code execution |
| `debug.*` | Sandbox introspection |

All system interaction goes through controlled `forge.*` surfaces. This is
especially important for third-party templates from a registry — templates
should be able to do anything a user would do inside the project directory,
and nothing else.

---

# v1 API Surface (complete reference)

```lua
-- Context
forge.project.name / .dir
forge.template.name / .dir

-- Arguments / Variables
forge.args(schema)
forge.vars.*

-- Rendering
forge.render(src)
forge.render_to(src, dst)
forge.render_dir(src)
forge.render_dir_to(src, dst)

-- Filesystem
forge.fs.exists(path)
forge.fs.mkdir(path)
forge.fs.write(path, content)
forge.fs.add(src, dst)
forge.fs.remove(path)

-- Execution
forge.exec(cmd, opts?)          -- returns { ok, code, stdout, stderr }

-- Logging
forge.log.info(msg)
forge.log.warn(msg)
forge.log.error(msg)
forge.log.success(msg)

-- Prompts
forge.prompt.input(msg, opts?)
forge.prompt.confirm(msg, opts?)
forge.prompt.select({ message, options, default? })

-- Strings
forge.str.camel / .pascal / .snake / .kebab / .upper / .lower

-- Environment
forge.env.*

-- Control
forge.abort(msg)

-- Hooks
forge.on_init(fn)
forge.on_complete(fn)
forge.on_error(fn)
```

---

# Future (post-v1)

- `forge.use("eslint")` — composable sub-templates
- `forge.is_dry_run()` + `--dry-run` CLI flag
- `forge.fs.read(path)` — read a file from the template directory
- `forge.fs.move(src, dst)` — rename or relocate a file
- `forge.fs.list(dir)` — list directory contents
- `forge.vars` `depends_on` — conditional variable visibility
- Template registry (`forge add user/template`)
