# Security Model

Templates execute inside a sandboxed Lua runtime. The runtime removes unsafe global APIs before template code runs.

| Removed global | Reason |
|---|---|
| `os.execute`, `os.exit` | Arbitrary command execution |
| `io.*` | Raw filesystem access |
| `require` | Loading external Lua modules |
| `load`, `loadfile`, `dofile` | Dynamic code execution |
| `debug.*` | Sandbox introspection |

All system interaction goes through controlled `forge.*` APIs. Third-party templates should be able to do normal project setup inside the output directory, and nothing else without explicit permissions.

## Permissions

Command execution, broader filesystem access, and environment access are controlled through `manifest.toml`.

See [manifest docs](../manifests.md) for `commands`, `programs`, and `permissions`.
