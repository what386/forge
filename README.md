# Forge

**Forge** is a Lua-powered project scaffolder.

It runs templates from local (`.forge/templates/`) or global (`~/.forge/templates/`) directories, renders files, prompts for inputs, and can run controlled command workflows.

---

## Features

- Scaffold projects with Lua templates
- Render templated files (`{{ expr }}`) and literal escaped blocks (`%{{...}}%`)
- Template trust model with checksum invalidation
- Fine-grained execution controls:
  - raw execution: `forge.exec` (`execution` + `commands`)
  - curated execution: `forge.prog.*` (`programs`)
- Config support (`~/.forge/config.toml`) including `[user]` defaults for template creation
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
forge new <template> <name> [--default]
forge list [--local|--global]
forge info <template>
forge create <name> [--global]
forge check <template> [--global]
```

### Trust commands

```bash
forge trust add <template> [--global]
forge trust remove <template>
forge trust list
```

### Config commands

```bash
forge config set user.name "Alice"
forge config set user.email "alice@example.com"
forge config get user.name
forge config list
forge config edit
```

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
```

Stdlib helpers exposed on `forge`:

- `forge.str.*`: `upper`, `lower`, `snake`, `kebab`, `pascal`, `camel`,
  `trim`, `split`, `starts_with`, `ends_with`, `join`
- `forge.table.*`: `merge`, `deep_merge`, `contains`, `keys`, `map`, `filter`
- `forge.path.*`: `join`, `basename`, `stem`, `ext`

For detailed API and manifest docs:

- `docs/lua_api.md`
- `docs/manifests.md`

---

## License

See project license files/repo policy.
