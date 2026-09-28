//! 用法错误：未声明字段 / group 与未知 ID 走退出码 2
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn unsupported_fields_and_unknown_ids_are_usage_errors() {
    let project = Project::new();
    project.configure();

    let (code, _) = project.fail(&[
        "add",
        "--kind",
        "project-note",
        "--group",
        "研究",
        "--set",
        "nope=1",
    ]);
    assert_eq!(code, 2, "an undeclared field is a usage error");

    let (code, _) = project.fail(&[
        "add",
        "--kind",
        "project-note",
        "--group",
        "不存在",
        "--set",
        "title=x",
    ]);
    assert_eq!(code, 2, "an undeclared group is a usage error");

    let (code, _) = project.fail(&["show", "WL-9999"]);
    assert_eq!(code, 2, "an unknown ID is a usage error");
}

#[test]
fn references_must_point_at_existing_records() {
    let project = Project::new();
    project.configure();
    project.add_work("已存在的事项", "todo");

    let (code, output) = project.fail(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=悬空父项",
        "--set",
        "status=todo",
        "--set",
        "parent=WL-9999",
    ]);
    assert_eq!(code, 2, "an unknown parent is refused: {output}");

    let (code, output) = project.fail(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=悬空依赖",
        "--set",
        "status=todo",
        "--set",
        "depends_on=WL-9999",
    ]);
    assert_eq!(code, 2, "an unknown dependency is refused: {output}");

    let (code, output) = project.fail(&["update", "WL-0001", "--set", "parent=WL-0001"]);
    assert_eq!(code, 2, "a record cannot depend on itself: {output}");
}

#[test]
fn depends_on_takes_a_list_and_parent_stays_single() {
    let project = Project::new();
    project.configure();
    let first = project.add_work("前置一", "todo");
    let second = project.add_work("前置二", "todo");

    let added = project.ok(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=依赖两项",
        "--set",
        "status=todo",
        "--set",
        &format!("depends_on=[{first}, {second}]"),
        "--json",
    ]);
    assert!(
        added.contains(&first) && added.contains(&second),
        "a bracketed list is written as several dependencies: {added}"
    );

    let (code, output) = project.fail(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=父项过多",
        "--set",
        "status=todo",
        "--set",
        &format!("parent={first},{second}"),
    ]);
    assert_eq!(code, 2, "`parent` takes a single record ID: {output}");
}

/// 读取端提前关闭管道时（`worklog show <ID> | head`），进程应当安静结束，
/// 而不是让 `println!` 抛出 Rust panic。
#[cfg(unix)]
#[test]
fn a_closed_pipe_ends_quietly_instead_of_panicking() {
    use std::io::Read;
    use std::process::Stdio;

    let project = Project::new();
    project.configure();
    // 正文远大于管道缓冲区，保证读取端退出后写入必然失败。
    let body = "x".repeat(256 * 1024);
    project.write(
        "worklog/items/WL-0001.md",
        &format!(
            "---\nid: WL-0001\nkind: project-note\ngroup: 研究\ntitle: 大记录\nphase: doing\n---\n\n## 目标\n\n{body}\n"
        ),
    );

    let mut child = project
        .command()
        .args(["show", "WL-0001", "--color", "never"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn the CLI");
    let mut stdout = child.stdout.take().expect("capture stdout");
    let mut head = [0_u8; 8];
    stdout.read_exact(&mut head).expect("read the first bytes");
    drop(stdout);

    let output = child.wait_with_output().expect("wait for the CLI");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty(),
        "a closed pipe must not print anything to stderr: {stderr}"
    );
    use std::os::unix::process::ExitStatusExt;
    assert_eq!(
        output.status.signal(),
        Some(libc::SIGPIPE),
        "the default SIGPIPE disposition ends the process silently: {:?}",
        output.status
    );
}
