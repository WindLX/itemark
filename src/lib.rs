//! Itemark：以稳定 ID 管理工作事项、事实与术语的本地 CLI。
//!
//! 模块按职责分层：
//!
//! - `domain`：ID、标量取值与头部字段等纯领域概念，不做 I/O。
//! - `record`：记录读写、现场索引与 Markdown 分节。
//! - `workspace`：项目配置、写入锁与写事务。
//! - `cli`：Clap 参数与子命令编排。
//!
//! 库入口供集成测试与二进制共用，测试只依赖 CLI 进程边界观察到的行为。

pub mod checks;
pub mod cli;
pub mod domain;
pub mod error;
pub mod i18n;
pub mod kind;
pub mod output;
pub mod record;
pub mod status;
pub mod style;
pub mod time;
pub mod view;
pub mod workspace;

pub use error::{Result, WorkspaceError};
