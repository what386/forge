# Prompts

Use `forge.prompt.*` for dynamic questions that cannot be declared statically in `forge.args`.

```lua
local org = forge.prompt.input("GitHub org", { default = "acme" })

local yes = forge.prompt.confirm("Enable Docker?", { default = false })

local db = forge.prompt.select({
    message = "Database",
    options = { "postgres", "sqlite", "mysql" },
    default = "postgres"
})
```

## Input Validation

```lua
local tag = forge.prompt.input("Image tag", {
    validate = function(v)
        return v:match("^[a-z0-9%-]+$") or "Only lowercase letters, numbers, hyphens"
    end
})
```
