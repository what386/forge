# Lua API Reference

## Context

```lua
forge.project.name
forge.project.dir
forge.template.name
forge.template.dir
```

## Arguments

```lua
forge.args(schema)
forge.vars.*
```

## Rendering

```lua
forge.render(src)
forge.render_to(src, dst)
forge.render_dir(src)
forge.render_dir_to(src, dst)
```

## Filesystem

```lua
forge.fs.exists(path)
forge.fs.mkdir(path)
forge.fs.write(path, content)
forge.fs.add(src, dst)
forge.fs.remove(path)
```

## Commands

```lua
forge.exec(cmd, opts?)
forge.prog.git.init()
forge.prog.git.add(...)
forge.prog.git.commit(message)
```

## Logging

```lua
forge.log.info(msg)
forge.log.warn(msg)
forge.log.error(msg)
forge.log.success(msg)
```

## Prompts

```lua
forge.prompt.input(msg, opts?)
forge.prompt.confirm(msg, opts?)
forge.prompt.select({ message, options, default? })
```

## Standard Library

```lua
forge.str.camel(s)
forge.str.pascal(s)
forge.str.snake(s)
forge.str.kebab(s)
forge.str.upper(s)
forge.str.lower(s)
forge.str.trim(s)
forge.str.split(s, sep)
forge.str.starts_with(s, prefix)
forge.str.ends_with(s, suffix)
forge.str.join(t, sep)

forge.table.merge(t1, t2)
forge.table.deep_merge(t1, t2)
forge.table.contains(t, value)
forge.table.keys(t)
forge.table.map(t, fn)
forge.table.filter(t, fn)

forge.path.join(...)
forge.path.basename(path)
forge.path.stem(path)
forge.path.ext(path)
```

## Environment

```lua
forge.env.*
```

## Control and Hooks

```lua
forge.abort(msg)
forge.on_init(fn)
forge.on_complete(fn)
forge.on_error(fn)
```
