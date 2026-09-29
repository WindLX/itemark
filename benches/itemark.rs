//! criterion 基准：写入路径、读取路径与引用校验的规模增长。
//!
//! 运行方式：
//!
//! ```text
//! cargo bench                 # 全部基准
//! cargo bench -- add          # 只跑写路径
//! cargo bench -- check        # 只跑引用校验
//! ```
//!
//! 基准直接调用库 API 并自建临时项目，因此测的是记录读写本身，不含 CLI 进程启动与
//! 参数解析。夹具与 `tests/common` 的项目形状保持一致（同样的 kind、group 与模板）。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use itemark::kind::render_template;
use itemark::record::Record;
use itemark::record::index::ItemIndex;
use itemark::workspace::Workspace;
use tempfile::TempDir;

/// 规模梯度：覆盖小项目到需要留意规模的项目。
const SIZES: [usize; 3] = [50, 200, 800];

const CONFIG: &str = r#"
language = "zh-CN"
root = "itemark-records"

[[groups]]
name = "产品"

[[kinds]]
name = "work"
description = "可继续推进的工作事项"
template = "templates/work.md"
required_sections = ["目标"]
completion_field = "status"
completion_values = ["done"]
fields = [
  { name = "title", type = "string", required = true },
  { name = "status", type = "enum", required = true, values = ["todo", "in_progress", "blocked", "done"] },
]
"#;

const TEMPLATE: &str = "---\nid: \"{{id}}\"\nkind: \"work\"\ngroup: \"{{group}}\"\ntitle: \"{{title}}\"\nstatus: \"{{status}}\"\n---\n## 目标\n\n";

fn fixture() -> TempDir {
    let dir = TempDir::new().expect("create benchmark project");
    fs::write(dir.path().join("itemark.toml"), CONFIG).expect("write itemark.toml");
    let templates = dir.path().join("itemark-records/templates");
    fs::create_dir_all(&templates).expect("create templates directory");
    fs::write(templates.join("work.md"), TEMPLATE).expect("write template");
    dir
}

fn config_path(dir: &Path) -> PathBuf {
    dir.join("itemark.toml")
}

/// 一条记录的最小正文，形状与模板渲染结果一致。
fn record_text(id: &str, title: &str, depends_on: Option<&str>) -> String {
    let mut text =
        format!("---\nid: {id}\nkind: work\ngroup: \"产品\"\ntitle: \"{title}\"\nstatus: todo\n");
    if let Some(previous) = depends_on {
        text.push_str(&format!("depends_on: [{previous}]\n"));
    }
    text.push_str("---\n\n## 目标\n\n基准夹具。\n\n");
    text
}

/// 铺 `count` 条记录，每条依赖前一条，让引用校验有真实的引用图可走。
fn seed(dir: &Path, count: usize) {
    let items = dir.join("itemark-records/items");
    fs::create_dir_all(&items).expect("create items directory");
    for index in 1..=count {
        let id = format!("IM-{index}");
        let previous = (index > 1).then(|| format!("IM-{}", index - 1));
        let text = record_text(&id, &format!("基准记录 {index}"), previous.as_deref());
        fs::write(items.join(format!("{id}.md")), text).expect("write record");
    }
}

/// 写路径：分配 ID、渲染模板、解析并写盘、更新内存索引。
fn bench_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("add");
    group.sample_size(20);
    group.bench_function("one_record", |b| {
        b.iter_batched(
            || {
                let dir = fixture();
                let workspace =
                    Workspace::open_locked(&config_path(dir.path()), None).expect("open workspace");
                (dir, workspace)
            },
            |(dir, mut workspace)| {
                let id = workspace.index().next_id().expect("allocate ID");
                let values = BTreeMap::from([
                    ("id".to_string(), id.clone()),
                    ("kind".to_string(), "work".to_string()),
                    ("group".to_string(), "产品".to_string()),
                    ("title".to_string(), "基准记录".to_string()),
                    ("status".to_string(), "todo".to_string()),
                ]);
                let rendered = render_template(TEMPLATE, &values);
                let target = workspace.config().items_dir().join(format!("{id}.md"));
                let record = Record::parse(&target, rendered).expect("parse record");
                let text = record.render();
                workspace
                    .transaction(|transaction| transaction.create(&id, &text))
                    .expect("create record");
                drop(dir);
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// 读路径：整项目扫描（`list` 的索引构建）与单条读取（`show`）。
fn bench_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("read");
    group.sample_size(30);
    for count in SIZES {
        let dir = fixture();
        seed(dir.path(), count);
        let workspace = Workspace::open(&config_path(dir.path()), None).expect("open workspace");
        let items = workspace.config().items_dir();
        let path = items.join(format!("IM-{count}.md"));

        group.throughput(Throughput::Elements(count as u64));
        group.bench_function(BenchmarkId::new("index_scan", count), |b| {
            b.iter(|| ItemIndex::scan(&items).expect("scan items"));
        });
        group.bench_function(BenchmarkId::new("show_one", count), |b| {
            b.iter(|| Record::read(&path).expect("read record"));
        });
    }
    group.finish();
}

/// 校验：全项目检查，引用校验在 IM-12 后应按 ID 建表查找。
fn bench_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("check");
    group.sample_size(30);
    for count in SIZES {
        let dir = fixture();
        seed(dir.path(), count);
        let workspace = Workspace::open(&config_path(dir.path()), None).expect("open workspace");
        assert!(
            itemark::checks::check_all(&workspace).is_ok(),
            "the fixture must pass its own checks"
        );

        group.throughput(Throughput::Elements(count as u64));
        group.bench_function(BenchmarkId::new("all_records", count), |b| {
            b.iter(|| itemark::checks::check_all(&workspace));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_add, bench_read, bench_check);
criterion_main!(benches);
