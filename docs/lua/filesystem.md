# Filesystem API

Filesystem access is sandboxed through `forge.fs`. Paths are relative to the project output directory.

```lua
forge.fs.exists(path)
forge.fs.mkdir(path)
forge.fs.write(path, content)
forge.fs.add(src, dst)
forge.fs.remove(path)
```

## Examples

```lua
if not forge.fs.exists("src") then
    forge.fs.mkdir("src")
end

forge.fs.write("src/main.rs", "fn main() {}\n")
forge.fs.add("assets/logo.png", "public/logo.png")
```

`forge.fs.add` copies a binary asset from the template's `files/` directory into the output directory.

`forge.fs.remove` is useful for cleanup in `forge.on_error`.
