# Getting Started

Forge templates live locally or globally and execute inside a sandboxed Lua runtime.

```bash
forge new <template> <name>
```

Example:

```bash
forge new webapp my-app
```

Forge will:

1. Locate the template.
2. Validate `manifest.toml`.
3. Execute `main.lua`.
4. Expose project metadata and APIs through `forge`.
5. Generate output into `./my-app/`.

## Template Structure

```text
.forge/templates/webapp/
├── main.lua
├── manifest.toml
└── files/
    ├── package.json.tpl
    ├── README.md.tpl
    └── src/
```

## Runtime Environment

Forge exposes one global table:

```lua
forge
```

Unsafe Lua globals such as `os.execute`, `io.open`, `require`, `load`, `dofile`, and public `debug.*` access are removed before template execution.

## Runtime Context

```lua
forge.project.name   -- name passed to `forge new`
forge.project.dir    -- absolute output directory

forge.template.name  -- template identifier
forge.template.dir   -- absolute template directory
```

## Metadata

Every template must include a `manifest.toml`. See [manifest docs](../manifests.md) for the complete schema.
