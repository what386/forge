use forge_te::lua::{ExecOptions, ExecResult, ExecRunner, LuaError, Runtime, RuntimeConfig};
use forge_te::templates::manifest::Permission;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

fn base_cfg() -> (RuntimeConfig, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let template_dir = tmp.path().join("template");
    let project_dir = tmp.path().join("out");
    fs::create_dir_all(template_dir.join("files")).expect("template files dir");
    fs::create_dir_all(&project_dir).expect("project dir");
    (
        RuntimeConfig {
            project_name: "my-app".to_string(),
            project_dir,
            template_name: "basic".to_string(),
            template_dir,
            ..RuntimeConfig::default()
        },
        tmp,
    )
}

#[test]
fn render_interpolates_template_vars() {
    let (cfg, _tmp) = base_cfg();
    let tpl = cfg.template_dir.join("files").join("README.md.tpl");
    fs::write(&tpl, "{{ forge.project.name }}").expect("write tpl");
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.render('README.md.tpl')").expect("write main");

    let mut rt = Runtime::new(cfg.clone());
    rt.run(main.to_string_lossy().as_ref()).expect("run");
    let out = fs::read_to_string(cfg.project_dir.join("README.md")).expect("read output");
    assert_eq!(out.trim(), "my-app");
}

#[test]
fn render_evaluates_call_site_locals() {
    let (cfg, _tmp) = base_cfg();
    let tpl = cfg.template_dir.join("files").join("math.txt.tpl");
    fs::write(&tpl, "{{ var + thing }} {{ label .. '-' .. thing }}").expect("write tpl");
    let main = cfg.template_dir.join("main.lua");
    fs::write(
        &main,
        r#"
        local var = 2
        local thing = 3
        local label = "sum"
        var = var + 10
        forge.render("math.txt.tpl")
        "#,
    )
    .expect("write main");

    let mut rt = Runtime::new(cfg.clone());
    rt.run(main.to_string_lossy().as_ref()).expect("run");
    let out = fs::read_to_string(cfg.project_dir.join("math.txt")).expect("read output");
    assert_eq!(out.trim(), "15 sum-3");
}

#[test]
fn render_evaluates_caller_upvalues() {
    let (cfg, _tmp) = base_cfg();
    let tpl = cfg.template_dir.join("files").join("upvalue.txt.tpl");
    fs::write(&tpl, "{{ prefix .. name .. count }}").expect("write tpl");
    let main = cfg.template_dir.join("main.lua");
    fs::write(
        &main,
        r#"
        local function build_renderer()
            local prefix = "project-"
            return function()
                local name = "forge-"
                local count = 7
                if prefix == "" then forge.abort("unreachable") end
                forge.render("upvalue.txt.tpl")
            end
        end
        local render_one = build_renderer()
        render_one()
        "#,
    )
    .expect("write main");

    let mut rt = Runtime::new(cfg.clone());
    rt.run(main.to_string_lossy().as_ref()).expect("run");
    let out = fs::read_to_string(cfg.project_dir.join("upvalue.txt")).expect("read output");
    assert_eq!(out.trim(), "project-forge-7");
}

#[test]
fn render_expression_uses_forge_helpers_and_nil_is_empty() {
    let (cfg, _tmp) = base_cfg();
    let tpl = cfg.template_dir.join("files").join("expr.txt.tpl");
    fs::write(
        &tpl,
        "{{ forge.str.upper(name) }}:{{ missing }}:{{ forge.project.name }}",
    )
    .expect("write tpl");
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "local name = 'forge'; forge.render('expr.txt.tpl')").expect("write main");

    let mut rt = Runtime::new(cfg.clone());
    rt.run(main.to_string_lossy().as_ref()).expect("run");
    let out = fs::read_to_string(cfg.project_dir.join("expr.txt")).expect("read output");
    assert_eq!(out.trim(), "FORGE::my-app");
}

#[test]
fn stdlib_scripts_are_exposed_on_forge_object() {
    let (cfg, _tmp) = base_cfg();
    let tpl = cfg.template_dir.join("files").join("stdlib.txt.tpl");
    fs::write(
        &tpl,
        "{{ forge.str.trim(raw) }}|{{ forge.str.join(parts, '-') }}|{{ forge.table.contains(parts, 'b') }}|{{ forge.path.ext(path) }}",
    )
    .expect("write tpl");
    let main = cfg.template_dir.join("main.lua");
    fs::write(
        &main,
        r#"
        local merged = forge.table.merge({ a = 1 }, { b = 2, a = 3 })
        if merged.a ~= 3 or merged.b ~= 2 then
            forge.abort("merge failed")
        end

        local deep = forge.table.deep_merge({ cfg = { one = 1 } }, { cfg = { two = 2 } })
        if deep.cfg.one ~= 1 or deep.cfg.two ~= 2 then
            forge.abort("deep_merge failed")
        end

        local keys = forge.table.keys({ x = 1, y = 2 })
        if #keys < 2 then
            forge.abort("keys failed")
        end

        local mapped = forge.table.map({ 1, 2, 3 }, function(v) return v * 2 end)
        if mapped[2] ~= 4 then
            forge.abort("map failed")
        end

        local filtered = forge.table.filter({ 1, 2, 3, 4 }, function(v) return v % 2 == 0 end)
        if filtered[1] ~= 2 or filtered[2] ~= 4 then
            forge.abort("filter failed")
        end

        local raw = "  hello  "
        local parts = forge.str.split("a,b,c", ",")
        if not forge.str.starts_with("forge", "for") then
            forge.abort("starts_with failed")
        end
        if not forge.str.ends_with("forge", "rge") then
            forge.abort("ends_with failed")
        end
        local path = forge.path.join("src", "main.rs")
        if forge.path.basename(path) ~= "main.rs" then
            forge.abort("basename failed")
        end
        if forge.path.stem(path) ~= "main" then
            forge.abort("stem failed")
        end
        forge.render("stdlib.txt.tpl")
        "#,
    )
    .expect("write main");

    let mut rt = Runtime::new(cfg.clone());
    rt.run(main.to_string_lossy().as_ref()).expect("run");
    let out = fs::read_to_string(cfg.project_dir.join("stdlib.txt")).expect("read output");
    assert_eq!(out.trim(), "hello|a-b-c|true|rs");
}

#[test]
fn fields_get_reads_configured_fields() {
    let (mut cfg, _tmp) = base_cfg();
    cfg.fields = BTreeMap::from([
        ("github.username".to_string(), "alice".to_string()),
        ("email".to_string(), "alice@example.com".to_string()),
    ]);
    let tpl = cfg.template_dir.join("files").join("fields.txt.tpl");
    fs::write(
        &tpl,
        "{{ forge.fields.get('github.username') }}|{{ forge.fields.get('email') }}",
    )
    .expect("write tpl");
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.render('fields.txt.tpl')").expect("write main");

    let mut rt = Runtime::new(cfg.clone());
    rt.run(main.to_string_lossy().as_ref()).expect("run");
    let out = fs::read_to_string(cfg.project_dir.join("fields.txt")).expect("read output");
    assert_eq!(out.trim(), "alice|alice@example.com");
}

#[test]
fn fields_get_errors_when_field_is_missing() {
    let (cfg, _tmp) = base_cfg();
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.fields.get('missing')").expect("write main");

    let mut rt = Runtime::new(cfg);
    let err = rt
        .run(main.to_string_lossy().as_ref())
        .expect_err("missing");
    assert!(err.to_string().contains("field not found: missing"));
}

#[test]
fn render_percent_literal_blocks_emit_raw_delimiters() {
    let (cfg, _tmp) = base_cfg();
    let tpl = cfg.template_dir.join("files").join("literal.txt.tpl");
    fs::write(
        &tpl,
        "run *args: %{{args}}% and %{{version}}% and {{ forge.project.name }}",
    )
    .expect("write tpl");
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.render('literal.txt.tpl')").expect("write main");

    let mut rt = Runtime::new(cfg.clone());
    rt.run(main.to_string_lossy().as_ref()).expect("run");
    let out = fs::read_to_string(cfg.project_dir.join("literal.txt")).expect("read output");
    assert_eq!(out.trim(), "run *args: {{args}} and {{version}} and my-app");
}

#[test]
fn render_expression_allows_right_brace_in_string_literal() {
    let (cfg, _tmp) = base_cfg();
    let tpl = cfg.template_dir.join("files").join("brace.txt.tpl");
    fs::write(&tpl, "{{ \"x}y\" }}").expect("write tpl");
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.render('brace.txt.tpl')").expect("write main");

    let mut rt = Runtime::new(cfg.clone());
    rt.run(main.to_string_lossy().as_ref()).expect("run");
    let out = fs::read_to_string(cfg.project_dir.join("brace.txt")).expect("read output");
    assert_eq!(out.trim(), "x}y");
}

#[test]
fn render_unterminated_percent_literal_block_errors() {
    let (cfg, _tmp) = base_cfg();
    let tpl = cfg.template_dir.join("files").join("bad_literal.txt.tpl");
    fs::write(&tpl, "broken %{{version}}").expect("write tpl");
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.render('bad_literal.txt.tpl')").expect("write main");

    let mut rt = Runtime::new(cfg);
    let err = rt.run(main.to_string_lossy().as_ref()).expect_err("error");
    assert!(err.to_string().contains("unterminated literal block"));
}

#[test]
fn sandbox_removes_dofile() {
    let (cfg, _tmp) = base_cfg();
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "if dofile ~= nil then forge.abort('unsafe') end").expect("write main");
    let mut rt = Runtime::new(cfg);
    rt.run(main.to_string_lossy().as_ref()).expect("run");
}

#[test]
fn sandbox_keeps_debug_private_from_templates_and_blocks() {
    let (cfg, _tmp) = base_cfg();
    let tpl = cfg.template_dir.join("files").join("debug.txt.tpl");
    fs::write(&tpl, "{{ debug == nil }}").expect("write tpl");
    let main = cfg.template_dir.join("main.lua");
    fs::write(
        &main,
        "if debug ~= nil then forge.abort('debug leaked') end; forge.render('debug.txt.tpl')",
    )
    .expect("write main");

    let mut rt = Runtime::new(cfg.clone());
    rt.run(main.to_string_lossy().as_ref()).expect("run");
    let out = fs::read_to_string(cfg.project_dir.join("debug.txt")).expect("read output");
    assert_eq!(out.trim(), "true");
}

#[test]
fn render_rejects_template_path_escape() {
    let (cfg, tmp) = base_cfg();
    fs::write(tmp.path().join("secret.txt.tpl"), "secret").expect("write secret");
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.render('../secret.txt.tpl')").expect("write main");

    let mut rt = Runtime::new(cfg);
    let err = rt.run(main.to_string_lossy().as_ref()).expect_err("escape");
    assert!(err.to_string().contains("path escapes template files dir"));
}

#[cfg(unix)]
#[test]
fn fs_write_rejects_symlink_parent_escape() {
    use std::os::unix::fs::symlink;

    let (cfg, tmp) = base_cfg();
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&outside).expect("outside");
    symlink(&outside, cfg.project_dir.join("link")).expect("symlink");

    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.fs.write('link/owned.txt', 'owned')").expect("write main");

    let mut rt = Runtime::new(cfg);
    let err = rt.run(main.to_string_lossy().as_ref()).expect_err("escape");
    assert!(err.to_string().contains("path escapes project dir"));
    assert!(!outside.join("owned.txt").exists());
}

#[cfg(unix)]
#[test]
fn exec_requires_declared_command() {
    let (mut cfg, _tmp) = base_cfg();
    cfg.permissions = vec![Permission::Execution];
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.exec({'sh', '-c', 'true'})").expect("write main");

    let mut rt = Runtime::new(cfg);
    let err = rt
        .run(main.to_string_lossy().as_ref())
        .expect_err("declared");
    assert!(err
        .to_string()
        .contains("command not declared in [requires].commands"));
}

#[cfg(unix)]
#[test]
fn exec_clears_undeclared_environment() {
    let (mut cfg, _tmp) = base_cfg();
    cfg.allowed_commands = vec!["sh".to_string()];
    cfg.permissions = vec![Permission::Execution];
    cfg.env_allowlist = vec!["PATH".to_string()];
    std::env::set_var("FORGE_SECRET_CLEAR_TEST_VALUE", "leaked");
    let main = cfg.template_dir.join("main.lua");
    fs::write(
        &main,
        r#"
        local out = forge.exec({'sh', '-c', 'printf "%s" "${FORGE_SECRET_CLEAR_TEST_VALUE:-}"'})
        forge.fs.write('env.txt', out.stdout)
        "#,
    )
    .expect("write main");

    let mut rt = Runtime::new(cfg.clone());
    rt.run(main.to_string_lossy().as_ref()).expect("run");
    let out = fs::read_to_string(cfg.project_dir.join("env.txt")).expect("read env");
    assert_eq!(out, "");
}

#[cfg(unix)]
#[test]
fn exec_can_inherit_environment_with_read_env_permission() {
    let (mut cfg, _tmp) = base_cfg();
    cfg.allowed_commands = vec!["sh".to_string()];
    cfg.permissions = vec![Permission::Execution, Permission::ReadEnv];
    std::env::set_var("FORGE_SECRET_INHERIT_TEST_VALUE", "visible");
    let main = cfg.template_dir.join("main.lua");
    fs::write(
        &main,
        r#"
        local out = forge.exec({'sh', '-c', 'printf "%s" "$FORGE_SECRET_INHERIT_TEST_VALUE"'})
        forge.fs.write('env.txt', out.stdout)
        "#,
    )
    .expect("write main");

    let mut rt = Runtime::new(cfg.clone());
    rt.run(main.to_string_lossy().as_ref()).expect("run");
    let out = fs::read_to_string(cfg.project_dir.join("env.txt")).expect("read env");
    assert_eq!(out, "visible\n");
}

#[cfg(unix)]
#[test]
fn exec_requires_execution_permission() {
    let (mut cfg, _tmp) = base_cfg();
    cfg.allowed_commands = vec!["sh".to_string()];
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.exec({'sh', '-c', 'true'})").expect("write main");

    let mut rt = Runtime::new(cfg);
    let err = rt
        .run(main.to_string_lossy().as_ref())
        .expect_err("requires permission");
    assert!(err
        .to_string()
        .contains("requires [requires].permissions to include execution"));
}

#[cfg(unix)]
#[test]
fn prog_git_init_requires_program_allowlist() {
    let (cfg, _tmp) = base_cfg();
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.prog.git.init()").expect("write main");

    let mut rt = Runtime::new(cfg);
    let err = rt
        .run(main.to_string_lossy().as_ref())
        .expect_err("requires program allowlist");
    assert!(err
        .to_string()
        .contains("program not declared in [requires].programs: git"));
}

#[cfg(unix)]
#[test]
fn prog_git_init_does_not_require_execution_permission() {
    let (mut cfg, _tmp) = base_cfg();
    cfg.allowed_programs = vec!["sh".to_string()];
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.prog.git.init()").expect("write main");

    let mut rt = Runtime::new(cfg);
    let err = rt
        .run(main.to_string_lossy().as_ref())
        .expect_err("requires git allowlist");
    assert!(err
        .to_string()
        .contains("program not declared in [requires].programs: git"));
}

#[cfg(unix)]
#[test]
fn prog_git_add_requires_arguments() {
    let (mut cfg, _tmp) = base_cfg();
    cfg.allowed_programs = vec!["git".to_string()];
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.prog.git.add()").expect("write main");

    let mut rt = Runtime::new(cfg);
    let err = rt
        .run(main.to_string_lossy().as_ref())
        .expect_err("must fail");
    assert!(err
        .to_string()
        .contains("forge.prog.git.add requires at least one argument"));
}

#[cfg(unix)]
#[test]
fn prog_git_commit_requires_message() {
    let (mut cfg, _tmp) = base_cfg();
    cfg.allowed_programs = vec!["git".to_string()];
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.prog.git.commit('  ')").expect("write main");

    let mut rt = Runtime::new(cfg);
    let err = rt
        .run(main.to_string_lossy().as_ref())
        .expect_err("must fail");
    assert!(err
        .to_string()
        .contains("forge.prog.git.commit requires a non-empty message"));
}

#[derive(Default)]
struct RecordingExecRunner {
    calls: Mutex<Vec<Vec<String>>>,
}

impl ExecRunner for RecordingExecRunner {
    fn run(
        &self,
        argv: &[String],
        _opts: &ExecOptions,
        _cwd: &Path,
        _env_allowlist: &[String],
        _inherit_env: bool,
    ) -> Result<ExecResult, LuaError> {
        self.calls.lock().expect("calls lock").push(argv.to_vec());
        Ok(ExecResult {
            ok: true,
            code: 0,
            ..ExecResult::default()
        })
    }
}

#[test]
fn prog_cargo_commands_emit_expected_argv() {
    let (mut cfg, _tmp) = base_cfg();
    let runner = Arc::new(RecordingExecRunner::default());
    cfg.allowed_programs = vec!["cargo".to_string()];
    cfg.exec = Some(runner.clone());
    let main = cfg.template_dir.join("main.lua");
    fs::write(
        &main,
        r#"
        forge.prog.cargo.init("--bin")
        forge.prog.cargo.new("cli-tool", "--bin")
        forge.prog.cargo.add("anyhow")
        forge.prog.cargo.build("--release")
        forge.prog.cargo.check()
        forge.prog.cargo.test("--all")
        forge.prog.cargo.run("--", "--help")
        forge.prog.cargo.fmt("--all")
        forge.prog.cargo.clippy("--all-targets")
        forge.prog.cargo.gen_lockfile()
        "#,
    )
    .expect("write main");

    let mut rt = Runtime::new(cfg);
    rt.run(main.to_string_lossy().as_ref()).expect("run");
    let calls = runner.calls.lock().expect("calls lock").clone();
    assert_eq!(
        calls,
        vec![
            vec!["cargo", "init", "--bin"],
            vec!["cargo", "new", "cli-tool", "--bin"],
            vec!["cargo", "add", "anyhow"],
            vec!["cargo", "build", "--release"],
            vec!["cargo", "check"],
            vec!["cargo", "test", "--all"],
            vec!["cargo", "run", "--", "--help"],
            vec!["cargo", "fmt", "--all"],
            vec!["cargo", "clippy", "--all-targets"],
            vec!["cargo", "generate-lockfile"],
        ]
    );
}

#[test]
fn prog_dotnet_commands_emit_expected_argv() {
    let (mut cfg, _tmp) = base_cfg();
    let runner = Arc::new(RecordingExecRunner::default());
    cfg.allowed_programs = vec!["dotnet".to_string()];
    cfg.exec = Some(runner.clone());
    let main = cfg.template_dir.join("main.lua");
    fs::write(
        &main,
        r#"
        forge.prog.dotnet.new("sln", { name = "my-app", format = "slnx" })
        forge.prog.dotnet.new("console", { name = "app", output = "src/app", no_restore = true })
        forge.prog.dotnet.sln_add("my-app.slnx", "src/app/app.csproj")
        forge.prog.dotnet.restore("my-app.slnx", { use_lock_file = true })
        "#,
    )
    .expect("write main");

    let mut rt = Runtime::new(cfg);
    rt.run(main.to_string_lossy().as_ref()).expect("run");
    let calls = runner.calls.lock().expect("calls lock").clone();
    assert_eq!(
        calls,
        vec![
            vec!["dotnet", "new", "sln", "--name", "my-app", "--format", "slnx"],
            vec![
                "dotnet",
                "new",
                "console",
                "--name",
                "app",
                "--output",
                "src/app",
                "--no-restore"
            ],
            vec!["dotnet", "sln", "my-app.slnx", "add", "src/app/app.csproj"],
            vec!["dotnet", "restore", "my-app.slnx", "--use-lock-file"],
        ]
    );
}

#[test]
fn prog_cargo_requires_program_allowlist() {
    let (cfg, _tmp) = base_cfg();
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.prog.cargo.check()").expect("write main");

    let mut rt = Runtime::new(cfg);
    let err = rt
        .run(main.to_string_lossy().as_ref())
        .expect_err("requires program allowlist");
    assert!(err
        .to_string()
        .contains("program not declared in [requires].programs: cargo"));
}

#[test]
fn prog_cargo_new_requires_arguments() {
    let (mut cfg, _tmp) = base_cfg();
    cfg.allowed_programs = vec!["cargo".to_string()];
    let main = cfg.template_dir.join("main.lua");
    fs::write(&main, "forge.prog.cargo.new()").expect("write main");

    let mut rt = Runtime::new(cfg);
    let err = rt
        .run(main.to_string_lossy().as_ref())
        .expect_err("must fail");
    assert!(err
        .to_string()
        .contains("forge.prog.cargo.new requires at least one argument"));
}
