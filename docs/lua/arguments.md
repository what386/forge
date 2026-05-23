# Arguments and Variables

Call `forge.args(schema)` once near the top of `main.lua`. Forge collects values from prompts or defaults before rendering begins.

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
        default = forge.env.USER
    }
})
```

## Supported Types

| Type | Prompt widget | Lua value |
|---|---|---|
| `string` | Text input | `string` |
| `number` | Text input | `number` |
| `boolean` | Confirm | `boolean` |
| `select` | Selection list | `string` |

`type` defaults to `"string"` when omitted. `options` is required for `select`.

`validate` is optional. Return `true` or nothing to pass; return a non-empty string to show that message and re-prompt.

## Accessing Values

```lua
forge.vars.language
forge.vars.port
forge.vars.git
forge.vars.author
```
