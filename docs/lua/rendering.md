# Rendering Files

Rendering reads from the template's `files/` directory and writes into the project output directory.

## Render a File

```lua
forge.render("README.md.tpl")
```

`.tpl` is stripped from the output path, so `README.md.tpl` writes `README.md`.

## Render to a Destination

```lua
forge.render_to("backend/go/go.mod.tpl", "backend/go.mod")
```

Use this when the output path depends on template logic.

## Render a Directory

```lua
forge.render_dir("frontend/react")
forge.render_dir_to("frontend/react", "client")
```

`.tpl` files are rendered and written without the `.tpl` extension. Other files are copied as-is.

## Template Interpolation

```text
{{ forge.project.name }}
{{ language }}
{{ frontend .. "-" .. backend }}
{{ port + 1 }}
{{ forge.str.pascal(forge.project.name) }}
```

Each block is one Lua expression. Forge evaluates it with locals and upvalues visible where `forge.render` or `forge.render_to` is called, plus sandboxed globals. `nil` renders as an empty string.

Use `%{{ ... }}%` when the output needs literal delimiters:

```text
%{{github.ref_name}}%  -- renders as {{github.ref_name}}
```

Keep conditionals and loops in `main.lua`; `.tpl` files only evaluate expressions.
