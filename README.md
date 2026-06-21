# Forge

**Forge** is a Lua-powered project scaffolder.

It runs templates from local (`.forge/templates/`), global (`~/.forge/templates/`), or package (`~/.forge/packages/`) directories, renders files, prompts for inputs, and can run controlled command workflows.

---

## Features

- Scaffold projects with Lua templates
- Render templated files (`{{ expr }}`) and literal escaped blocks (`%{{...}}%`)
- Template trust model with checksum invalidation
- Fine-grained execution controls:
  - raw execution: `forge.exec` (`execution` + `commands`)
  - curated execution: `forge.prog.*` (`programs`)
- Config support (`~/.forge/config.toml`) including `[user]` defaults for template creation and `[templates]` default scope
- Template validation before execution

---

## Installation

### With Cargo

```bash
cargo install --path .
```

Or run directly in this repo:

```bash
cargo run -- <command>
```

---

## Quick Start

### List templates

```bash
forge list
forge list --local
forge list --global
```

### Create a project from a template

```bash
forge new rust my-app
```

Use defaults for all prompts:

```bash
forge new rust my-app --default
```

### Inspect and validate templates

```bash
forge info rust
forge check rust
```

---

## Command Overview

### Project and template commands

```bash
forge new <template> <name> [--local|--global] [--default]
forge list [--local|--global]
forge info <template> [--local|--global]
forge create <name> [--local|--global]
forge remove <template>... [--local|--global]
forge check <template> [--local|--global]
```

### Trust commands

```bash
forge trust add <template> [--local|--global]
forge trust remove <template> [--local|--global]
forge trust list
```

### Package commands

```bash
forge package probe <repo>
forge package install <repo> <template>...
forge package remove [template]...
forge package update [template]...
forge package list
```

### Config commands

```bash
forge config set user.name "Alice"
forge config set user.email "alice@example.com"
forge config set templates.default_scope global
forge config get user.name
forge config list
forge config edit
```

`templates.default_scope` accepts `local` or `global`. Unscoped template commands
try that scope first and then package-installed templates. `forge list` shows
local, global, and package-installed templates unless filtered with `--local` or
`--global`.

### Field commands

```bash
forge fields set github.username=alice
forge fields get github.username
forge fields clear github.username
forge fields list
```

Fields are stored in `~/.forge/fields.json` and exposed to templates with
`forge.fields.get("github.username")`.

---

## Permissions and Trust

Templates can declare requirements in `manifest.toml`:

```toml
[requires]
commands = ["git", "npm"]
programs = ["git"]
permissions = ["execution", "escape_cwd", "network", "read_env"]
```

- `commands`: allowlist for raw `forge.exec`
- `programs`: allowlist for curated `forge.prog.*` APIs
- `permissions`: elevated capability flags

When a template requests elevated capabilities and is not trusted, Forge shows a summary and prompts:

- `y`: trust and persist checksum in `~/.forge/trust.json`
- `n`: run once (no trust persisted)
- `q`: abort

If template contents change later, trust is invalidated automatically.

---

## Template Authoring Notes

Template layout:

```text
.forge/templates/<name>/
  manifest.toml
  main.lua
  files/
```

Remote template package repositories use the same template entries in a root
`index.json` file:

```json
{
  "version": 1,
  "templates": [
    { "name": "rust", "path": "templates/rust" }
  ]
}
```

- `.tpl` files in `files/` are rendered and `.tpl` is stripped
- non-`.tpl` files are copied as-is
- expression interpolation in `.tpl`:
  - `{{ forge.project.name }}`
  - `%{{github.ref_name}}%` to emit literal `{{github.ref_name}}`

Curated program API example:

```lua
forge.prog.git.init()
forge.prog.git.add("-A")
forge.prog.git.commit("chore: initial scaffold")
forge.prog.cargo.init("--bin")
forge.prog.cargo.add("anyhow")
forge.prog.cargo.check()
forge.prog.cargo.gen_lockfile()
```

Stdlib helpers exposed on `forge`:

- `forge.str.*`: `upper`, `lower`, `snake`, `kebab`, `pascal`, `camel`,
  `trim`, `split`, `starts_with`, `ends_with`, `join`
- `forge.table.*`: `merge`, `deep_merge`, `contains`, `keys`, `map`, `filter`
- `forge.path.*`: `join`, `basename`, `stem`, `ext`

For detailed API and manifest docs:

- `docs/lua/index.md`
- `docs/manifests.md`

---

## License

See project license files/repo policy.
