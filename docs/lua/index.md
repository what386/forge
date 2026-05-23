# Lua Template API

Forge templates are Lua programs that generate projects through one global table:

```lua
forge
```

Use these pages as the template-authoring guide. They are split into small files so they can be rendered directly by a docs site generator.

## Start Here

- [Getting started](getting-started.md): template layout, execution flow, and context values.
- [Arguments and variables](arguments.md): collect user input with `forge.args`.
- [Rendering files](rendering.md): render `.tpl` files and directories.

## Runtime APIs

- [Filesystem](filesystem.md): write, copy, create, remove, and check output files.
- [Commands](commands.md): run external commands and use curated program APIs.
- [Prompts](prompts.md): ask dynamic questions during execution.
- [Standard library](stdlib.md): string, table, and path helpers.
- [Hooks and control](hooks.md): lifecycle hooks, logging, and aborting.
- [Security model](security.md): sandbox rules and permission boundaries.

## Reference

- [API reference](reference.md): compact list of all exposed `forge.*` APIs.
