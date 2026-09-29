//! 归档与取消归档：归档独立于业务状态，记录仍按稳定 ID 查询。
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn archive_keeps_ids_status_references_and_can_be_included_in_queries() {
    let project = Project::new();
    project.configure();
    let id = project.add_work("待归档事项", "in_progress");
    let reference = project.add_work("引用归档事项", "todo");
    project.ok(&["update", &reference, "--set", &format!("depends_on={id}")]);
    for item in [&id, &reference] {
        project.ok(&[
            "update",
            item,
            "--section",
            "目标=归档行为测试目标",
            "--section",
            "验收=真实CLI检查通过",
        ]);
    }
    let mut items_path = project.item_file(&id);
    let filename = items_path
        .file_name()
        .expect("record filename")
        .to_string_lossy()
        .into_owned();
    let mut archived_path = project
        .path()
        .join("itemark-records/archive")
        .join(filename);

    let archive_result = project.ok(&["archive", &id, "--json"]);
    let archive_result: serde_json::Value =
        serde_json::from_str(&archive_result).expect("valid archive JSON");
    assert_eq!(archive_result["archived"][0], id);

    assert!(!items_path.exists());
    assert!(archived_path.exists());
    let original_archived_path = archived_path.clone();
    let shown = project.ok(&["show", &id, "--json"]);
    let shown: serde_json::Value = serde_json::from_str(&shown).expect("valid show JSON");
    assert_eq!(shown["id"], id);
    assert_eq!(shown["status"], "in_progress");
    assert_eq!(shown["lifecycle"], "active");
    assert!(project.ok(&["check", &id]).contains("检查通过"));
    assert!(project.ok(&["check", &reference]).contains("检查通过"));

    let renamed = project.ok(&["update", &id, "--set", "title=归档后改名", "--json"]);
    let renamed: serde_json::Value = serde_json::from_str(&renamed).expect("valid update JSON");
    archived_path = PathBuf::from(renamed["path"].as_str().expect("updated record path"));
    let archive_root = fs::canonicalize(project.path().join("itemark-records/archive"))
        .expect("canonical archive directory");
    let canonical_archived_path = fs::canonicalize(&archived_path).expect("canonical record path");
    assert!(
        canonical_archived_path.starts_with(&archive_root),
        "updated record path must remain under the archive root: {canonical_archived_path:?}"
    );
    assert!(!original_archived_path.exists());
    items_path = project
        .path()
        .join("itemark-records/items")
        .join(archived_path.file_name().expect("renamed record filename"));

    let listed = project.ok(&["list", "--json"]);
    assert!(!item_ids(&listed).contains(&id), "{listed}");
    let all = project.ok(&["list", "--all", "--json"]);
    assert!(
        !item_ids(&all).contains(&id),
        "--all only includes dropped records"
    );
    let included = project.ok(&["list", "--include-archived", "--json"]);
    assert!(item_ids(&included).contains(&id));
    let archived = project.ok(&["list", "--archived", "--json"]);
    assert_eq!(item_ids(&archived), [id.as_str()]);
    let searched = project.ok(&["search", "归档后改名", "--json"]);
    assert!(!item_ids(&searched).contains(&id));
    let included_search = project.ok(&["search", "归档后改名", "--include-archived", "--json"]);
    assert!(item_ids(&included_search).contains(&id));
    let overview = project.ok(&["summary", "--json"]);
    assert!(!json_strings(&overview, "sources").contains(&id));
    let included_overview = project.ok(&["summary", "--include-archived", "--json"]);
    assert!(json_strings(&included_overview, "sources").contains(&id));

    let unarchive_result = project.ok(&["unarchive", &id, "--json"]);
    let unarchive_result: serde_json::Value =
        serde_json::from_str(&unarchive_result).expect("valid unarchive JSON");
    assert_eq!(unarchive_result["unarchived"][0], id);
    assert!(!archived_path.exists());
    assert!(items_path.exists());
    let restored = project.ok(&["show", &id, "--json"]);
    let restored: serde_json::Value = serde_json::from_str(&restored).expect("valid show JSON");
    assert_eq!(restored["status"], "in_progress");
    assert!(item_ids(&project.ok(&["list", "--json"])).contains(&id));
    assert!(project.ok(&["check", &reference]).contains("检查通过"));
}

#[test]
fn archive_batches_preflight_every_id_and_destination_before_moving() {
    let project = Project::new();
    project.configure();
    let first = project.add_work("第一条", "todo");
    let second = project.add_work("第二条", "blocked");
    let first_path = project.item_file(&first);
    let second_path = project.item_file(&second);
    let archive_dir = project.path().join("itemark-records/archive");
    let second_archive_path =
        archive_dir.join(second_path.file_name().expect("second record filename"));

    fs::create_dir_all(&second_archive_path).expect("create conflicting archive destination");
    let (_, error) = project.fail(&["archive", &first, &second]);
    assert!(error.contains("已存在"), "{error}");
    assert!(
        first_path.exists(),
        "no earlier record moved before preflight finished"
    );
    assert!(second_path.exists());
    fs::remove_dir(&second_archive_path).expect("remove conflict directory");

    let (_, missing) = project.fail(&["archive", &first, "IM-404"]);
    assert!(missing.contains("IM-404"), "{missing}");
    assert!(first_path.exists(), "missing IDs are checked before moving");

    project.ok(&["archive", &first, &second]);
    assert!(!first_path.exists());
    assert!(!second_path.exists());
    let first_archive_path =
        archive_dir.join(first_path.file_name().expect("first record filename"));
    assert!(first_archive_path.exists());

    let (_, missing) = project.fail(&["unarchive", &first, "IM-404"]);
    assert!(missing.contains("IM-404"), "{missing}");
    assert!(
        first_archive_path.exists(),
        "missing IDs do not partially unarchive"
    );

    let first_items_collision = project
        .path()
        .join("itemark-records/items")
        .join(first_path.file_name().expect("first record filename"));
    fs::create_dir_all(&first_items_collision).expect("create conflicting items destination");
    let (_, error) = project.fail(&["unarchive", &first, &second]);
    assert!(error.contains("已存在"), "{error}");
    assert!(
        first_archive_path.exists(),
        "no earlier record moved before preflight finished"
    );
    fs::remove_dir(&first_items_collision).expect("remove conflict directory");

    project.ok(&["unarchive", &first, &second]);
    assert!(first_path.exists());
    assert!(second_path.exists());
}

fn json_strings(json: &str, key: &str) -> Vec<String> {
    serde_json::from_str::<serde_json::Value>(json).expect("valid JSON output")[key]
        .as_array()
        .expect("array of strings")
        .iter()
        .map(|value| value.as_str().expect("string value").to_string())
        .collect()
}

fn item_ids(json: &str) -> Vec<String> {
    serde_json::from_str::<serde_json::Value>(json).expect("valid JSON output")["items"]
        .as_array()
        .expect("array of records")
        .iter()
        .map(|item| item["id"].as_str().expect("record ID").to_string())
        .collect()
}
