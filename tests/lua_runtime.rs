use forge::lua::{Runtime, RuntimeConfig};
use forge::templates::manifest::Permission;
use std::fs;

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
    let (cfg, _tmp) = base_cfg();
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
    cfg.permissions = vec![Permission::ReadEnv];
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
