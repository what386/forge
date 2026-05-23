# Hooks, Logging, and Control

## Logging

```lua
forge.log.info("Creating project structure")
forge.log.warn("git not found, skipping init")
forge.log.error("Config is invalid")
forge.log.success("Project ready")
```

`forge.log.error` does not abort. Use `forge.abort()` for that.

## Abort

```lua
forge.abort("Directory is not empty")
```

`forge.abort` terminates execution with a clean user-facing message. Prefer it over `error()` when the message should be read by the user.

If an `on_error` hook is registered, it runs before the process exits.

## Lifecycle Hooks

```lua
forge.on_init(function()
    forge.log.info("Starting setup")
end)

forge.on_complete(function()
    forge.log.success("Done")
end)

forge.on_error(function(err)
    forge.log.warn("Error: " .. err)
    forge.fs.remove(forge.project.dir)
end)
```

Registering a hook twice replaces the previous handler. Only one handler per hook is supported.
