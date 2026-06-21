# Commands

Templates must declare `execution` in `[requires].permissions` before calling `forge.exec`.

Always pass commands as an argument table. Shell string execution is not supported.

```lua
local result = forge.exec({ "git", "init" })
```

## Return Value

```lua
result.ok      -- true when exit code is 0
result.code    -- raw exit code
result.stdout  -- captured stdout
result.stderr  -- captured stderr
```

## Options

```lua
forge.exec({ "npm", "install" }, {
    cwd         = "frontend",
    allow_fail  = false,
    passthrough = false,
    on_stdout   = function(line) end,
    on_stderr   = function(line) end,
})
```

`cwd` is relative to the project output directory.

`allow_fail = true` lets the template inspect a failed command instead of aborting.

`passthrough = true` streams output directly to the terminal. Captured stdout and stderr will be empty.

`on_stdout` and `on_stderr` are called line-by-line as output arrives. They are mutually exclusive with `passthrough`.

## Curated Program API

Use `forge.prog.*` wrappers for curated command families. These use `[requires].programs` instead of raw `execution` permission.

```lua
forge.prog.git.init()
forge.prog.git.add("-A")
forge.prog.git.commit("chore: initial scaffold")

forge.prog.cargo.init("--bin")
forge.prog.cargo.add("anyhow")
forge.prog.cargo.check()
forge.prog.cargo.gen_lockfile()
```

## Detecting Tools

```lua
local bun = forge.exec({ "which", "bun" }, { allow_fail = true })
local pm = bun.ok and "bun" or "npm"
forge.exec({ pm, "install" })
```
