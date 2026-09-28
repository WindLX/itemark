//! 统一的命令结果与错误类型。
//!
//! 错误分两类：
//! - 用法错误（退出状态 2）：命令参数、字段名或子命令用法不正确。
//! - 运行错误（退出状态 1）：配置、记录、引用或并发冲突导致的失败。
//!
//! 内部诊断与 debug 文案使用英文；面向使用者的呈现由输出层按项目语言生成。

use std::path::Path;

pub type Result<T> = std::result::Result<T, WorkspaceError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Usage,
    Runtime,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct WorkspaceError {
    kind: ErrorKind,
    message: String,
}

impl WorkspaceError {
    pub fn usage(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Usage,
            message: message.into(),
        }
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Runtime,
            message: message.into(),
        }
    }

    /// 在错误信息前补充文件或目录位置。
    #[must_use]
    pub fn at(self, path: &Path) -> Self {
        Self {
            kind: self.kind,
            message: format!("{}: {}", path.display(), self.message),
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self.kind {
            ErrorKind::Usage => 2,
            ErrorKind::Runtime => 1,
        }
    }
}

impl From<std::io::Error> for WorkspaceError {
    fn from(error: std::io::Error) -> Self {
        Self::runtime(error.to_string())
    }
}

impl From<serde_json::Error> for WorkspaceError {
    fn from(error: serde_json::Error) -> Self {
        Self::runtime(error.to_string())
    }
}

/// 读取失败时补齐路径与原因。
pub fn read_error(path: &Path, error: std::io::Error) -> WorkspaceError {
    WorkspaceError::runtime(format!("cannot read file: {error}")).at(path)
}
