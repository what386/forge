# Standard Library

Forge exposes a small Lua standard library for common template work.

## String Utilities

Case helpers accept camelCase, PascalCase, snake_case, or kebab-case input.

```lua
forge.str.camel("hello-world")        --> "helloWorld"
forge.str.pascal("hello-world")       --> "HelloWorld"
forge.str.snake("hello-world")        --> "hello_world"
forge.str.kebab("helloWorld")         --> "hello-world"
forge.str.upper("hello")              --> "HELLO"
forge.str.lower("HELLO")              --> "hello"
forge.str.trim("  hi  ")              --> "hi"
forge.str.split("a,b,c", ",")         --> { "a", "b", "c" }
forge.str.starts_with("forge", "for") --> true
forge.str.ends_with("forge", "rge")   --> true
forge.str.join({ "a", "b" }, "-")     --> "a-b"
```

## Table Utilities

```lua
forge.table.merge({ a = 1 }, { a = 2, b = 3 })
forge.table.deep_merge({ cfg = { a = 1 } }, { cfg = { b = 2 } })
forge.table.contains({ "a", "b" }, "b")
forge.table.keys({ a = 1, b = 2 })
forge.table.map({ 1, 2, 3 }, function(v) return v * 2 end)
forge.table.filter({ 1, 2, 3, 4 }, function(v) return v % 2 == 0 end)
```

`merge` is shallow and the second table wins on conflict. `deep_merge` recursively merges nested tables.

`map` and `filter` call the function as `fn(value, key)`.

## Path Utilities

```lua
forge.path.join("src", "main.rs")
forge.path.basename("src/main.rs")  --> "main.rs"
forge.path.stem("main.rs")          --> "main"
forge.path.ext("main.rs")           --> "rs"
```
