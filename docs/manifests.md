# Forge `manifest.toml` Specification

## Overview

Every Forge template must include a `manifest.toml` at its root. Forge reads
this file before executing `main.lua` — if the manifest is missing, malformed,
or fails validation, the template will not run.

---

# Location

```text
.forge/templates/<template-name>/manifest.toml
```

---

# Full Example

```toml
[package]
name              = "fullstack"
version           = "1.0.0"
description       = "A fullstack web app with a choice of frontend, backend, and database"
min_forge_version = "1.0.0"
repository        = "https://github.com/alice/forge-templates"

[author]
name  = "Alice"
email = "alice@example.com"
url   = "https://github.com/alice"

[tags]
values = ["fullstack", "web", "docker"]

[requires]
commands    = ["git", "docker"]
permissions = ["network"]
```

---

# Fields

All identity fields live under `[package]`.

## `[package].name`
**Required.** The template identifier. Must match the directory name under
`.forge/templates/`. Lowercase letters, numbers, and hyphens only.

```toml
[package]
name = "fullstack"
```

---

## `[package].version`
**Required.** Semantic version of the template itself. Increment this when
the template's behavior or file output changes in a meaningful way.

```toml
[package]
version = "1.0.0"
```

---

## `[package].description`
**Required.** A short, human-readable description of what the template
generates. Used in template listings and registry search.

```toml
[package]
description = "A fullstack web app with a choice of frontend, backend, and database"
```

---

## `[package].min_forge_version`
**Required.** The minimum version of Forge required to run this template.
Forge will refuse to execute the template and print a clear error if the
running version is older.

```toml
[package]
min_forge_version = "1.0.0"
```

---

## `[package].repository`
**Optional.** URL of the template's source repository. Used for attribution
and will be surfaced by the future template registry.

```toml
[package]
repository = "https://github.com/alice/forge-templates"
```

---

## `[author]`
**Optional.** Information about the template author. All subfields are optional.

```toml
[author]
name  = "Alice"
email = "alice@example.com"
url   = "https://github.com/alice"
```

| Field   | Description                        |
|---------|------------------------------------|
| `name`  | Author's display name              |
| `email` | Contact email                      |
| `url`   | Homepage, GitHub profile, or repo  |

---

## `[tags]`
**Optional.** A list of keywords used for template discovery and search.
Lowercase only. Used by the future template registry.

```toml
[tags]
values = ["fullstack", "web", "docker"]
```

---

# Validation Rules

Forge validates the manifest before execution. The following will produce a
clear error and prevent the template from running:

- `[package]` section is missing
- `name`, `version`, `description`, or `min_forge_version` is missing from `[package]`
- `name` contains uppercase letters, spaces, or special characters other than `-`
- `name` does not match the template's directory name
- `version` or `min_forge_version` is not a valid semantic version (`MAJOR.MINOR.PATCH`)
- `min_forge_version` is higher than the running Forge version
- Any unknown top-level key is present (strict parsing — no silent ignoring of typos)

---

# Future Fields

## `[requires]`
**Optional.** Declares external dependencies and elevated permissions the
template needs. Forge validates this section before running `main.lua`.

### `[requires].commands`
A list of binaries that must exist on the host before the template runs.
Forge checks each with a `which`-equivalent call and aborts with a clear
message if any are missing.

```toml
[requires]
commands = ["git", "docker"]
```

### `[requires].permissions`
A list of elevated capabilities the template needs beyond the default sandbox.
Forge will display these to the user and ask for confirmation before running.

```toml
[requires]
permissions = ["escape_cwd", "network", "read_env"]
```

| Permission   | What it unlocks |
|--------------|-----------------|
| `escape_cwd` | `forge.fs` and `forge.exec` `cwd` may reference paths outside the output directory |
| `network`    | Commands that make network requests (signals intent; not enforced at the syscall level in v1) |
| `read_env`   | Access to environment variables beyond the default allowlist (`HOME`, `USER`, `PATH`, `SHELL`) |

Forge will print a permission summary and prompt the user to confirm before
executing any template that declares permissions:

```
Template "fullstack" is requesting elevated permissions:
  • escape_cwd  — may access paths outside the project directory
  • network     — may make network requests
Proceed? (y/n)
```

Unknown permission strings are treated as a validation error — not silently ignored.
