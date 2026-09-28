//! 并发：多进程添加分配唯一 ID，并发更新不静默丢失
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn concurrent_adds_allocate_unique_ids() {
    let project = Project::new();
    project.configure();

    let titles: Vec<String> = (0..4).map(|index| format!("并发记录{index}")).collect();
    let children: Vec<_> = titles
        .iter()
        .map(|title| {
            let assignment = format!("title={title}");
            project
                .command()
                .args([
                    "add",
                    "--kind",
                    "project-note",
                    "--group",
                    "研究",
                    "--set",
                    &assignment,
                    "--set",
                    "phase=doing",
                    "--json",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("spawn a concurrent add")
        })
        .collect();

    let mut ids = Vec::new();
    for child in children {
        let output = child.wait_with_output().expect("wait for the add");
        assert!(
            output.status.success(),
            "concurrent add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        ids.extend(ids_in(&String::from_utf8_lossy(&output.stdout)));
    }

    ids.sort();
    ids.dedup();
    assert_eq!(
        ids.len(),
        4,
        "every concurrent add gets its own ID: {ids:?}"
    );
    let files = fs::read_dir(project.path().join("worklog/items"))
        .expect("read items directory")
        .count();
    assert_eq!(files, 4, "no record is lost");
}

#[test]
fn concurrent_updates_never_silently_lose_progress() {
    let project = Project::new();
    project.configure();
    let id = project.add_note("并发更新", "研究");

    let logs: Vec<_> = ["甲", "乙"]
        .iter()
        .enumerate()
        .map(|(index, text)| {
            project
                .command()
                .args([
                    "log",
                    &id,
                    text,
                    "--date",
                    &format!("2024-0{}-01", index + 1),
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("spawn a concurrent log")
        })
        .collect();

    let results: Vec<_> = logs
        .into_iter()
        .map(|child| child.wait_with_output().expect("wait for the log"))
        .collect();
    let body = project.read("worklog/items/WL-0001.md");

    for (text, output) in ["甲", "乙"].iter().zip(&results) {
        assert!(
            body.contains(text) || !output.status.success(),
            "`{text}` is either written or explicitly reported as a conflict"
        );
    }
    assert!(
        results.iter().any(|output| output.status.success()),
        "at least one concurrent update gets through"
    );
    assert!(body.contains("并发更新"), "the record itself survives");
}
