-- .forge/templates/fullstack/main.lua
-- A fullstack web app: frontend framework + backend language + database + optional services.

-- ── Hooks ─────────────────────────────────────────────────────────────────────

forge.on_init(function()
    forge.log.info("Scaffolding " .. forge.project.name)
end)

forge.on_error(function()
    forge.log.warn("Rolling back — removing partial output")
    forge.fs.remove(forge.project.dir)
end)

forge.on_complete(function()
    forge.log.success("Created " .. forge.project.name)
    forge.log.info("")
    forge.log.info("  cd " .. forge.project.name)
    forge.log.info("  npm run dev")
end)

-- ── Args ──────────────────────────────────────────────────────────────────────

forge.args({
    frontend = {
        prompt  = "Frontend framework",
        type    = "select",
        options = { "react", "svelte", "vue" },
        default = "react",
    },
    backend = {
        prompt  = "Backend language",
        type    = "select",
        options = { "go", "rust", "node" },
        default = "go",
    },
    database = {
        prompt  = "Database",
        type    = "select",
        options = { "postgres", "sqlite", "mysql" },
        default = "postgres",
    },
    docker = {
        prompt  = "Add Docker Compose?",
        type    = "boolean",
        default = true,
    },
    git = {
        prompt  = "Initialize git repo?",
        type    = "boolean",
        default = true,
    },
    port_frontend = {
        prompt   = "Frontend port",
        type     = "number",
        default  = 5173,
        validate = function(v)
            return (v > 1024 and v < 65536) or "Must be between 1025 and 65535"
        end,
    },
    port_backend = {
        prompt   = "Backend port",
        type     = "number",
        default  = 8080,
        validate = function(v)
            return (v > 1024 and v < 65536) or "Must be between 1025 and 65535"
        end,
    },
})

-- ── Derived values ────────────────────────────────────────────────────────────

local fe = forge.vars.frontend
local be = forge.vars.backend
local db = forge.vars.database

-- ── Package manager detection ─────────────────────────────────────────────────

local pm
if   forge.exec({ "which", "bun"  }, { allow_fail = true }).ok then pm = "bun"
elseif forge.exec({ "which", "pnpm" }, { allow_fail = true }).ok then pm = "pnpm"
else pm = "npm"
end

forge.log.info("Using package manager: " .. pm)

-- ── Root files ────────────────────────────────────────────────────────────────

forge.render("README.md.tpl")
forge.render(".env.example.tpl")

-- ── Frontend ──────────────────────────────────────────────────────────────────

forge.log.info("Setting up " .. fe .. " frontend")

forge.render_dir("frontend/" .. fe)
forge.render("frontend/vite.config.tpl")
forge.render("frontend/package.json.tpl")

-- ── Backend ───────────────────────────────────────────────────────────────────

forge.log.info("Setting up " .. be .. " backend")

forge.render_dir("backend/" .. be)

if be == "go" then
    forge.render("backend/go/go.mod.tpl")
elseif be == "rust" then
    forge.render("backend/rust/Cargo.toml.tpl")
elseif be == "node" then
    forge.render("backend/node/package.json.tpl")
    forge.render("backend/node/tsconfig.json.tpl")
end

-- ── Database ──────────────────────────────────────────────────────────────────

forge.log.info("Configuring " .. db)

forge.render_to("db/" .. db .. ".tpl", "backend/config/database.conf")

if db == "postgres" or db == "mysql" then
    forge.log.warn(db .. " requires a running server — see .env.example")
end

-- ── Docker ────────────────────────────────────────────────────────────────────

if forge.vars.docker then
    forge.render("docker/compose.tpl")
    forge.render_to("docker/frontend.Dockerfile.tpl", "frontend/Dockerfile")
    forge.render_to("docker/backend.Dockerfile.tpl",  "backend/Dockerfile")
    forge.log.success("Docker Compose configured")
end

-- ── Git ───────────────────────────────────────────────────────────────────────

if forge.vars.git then
    forge.render(".gitignore.tpl")

    local r = forge.exec({ "git", "init" }, { allow_fail = true })
    if r.ok then
        forge.exec({ "git", "add", "-A" })
        forge.exec({ "git", "commit", "-m", "chore: initial scaffold via forge" })
        forge.log.success("Git repo initialized")
    else
        forge.log.warn("git init failed — skipping")
    end
end

-- ── Install dependencies ──────────────────────────────────────────────────────

forge.log.info("Installing frontend dependencies")

local frontend_install = forge.exec(
    { pm, "install" },
    { cwd = "frontend", allow_fail = true }
)
if not frontend_install.ok then
    forge.log.warn("Install failed — run `" .. pm .. " install` in frontend/ manually")
    forge.log.warn(frontend_install.stderr)
end

if be == "node" then
    forge.log.info("Installing backend dependencies")
    forge.exec({ pm, "install" }, { cwd = "backend" })
end
