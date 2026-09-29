//! CLI 写操作的协作锁。
//!
//! 使用标准库文件锁，不额外引入锁 crate。锁只约束遵守它的 CLI 进程；文本编辑器
//! 和其他外部进程不参与该锁，只能靠写入前的内容比对发现部分冲突。

use std::fs::{File, OpenOptions};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::error::{Result, WorkspaceError};

/// 等待锁的最长时间；超过则明确报冲突，不静默继续。
const LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const RETRY_INTERVAL: Duration = Duration::from_millis(20);

/// 对项目锁文件持有排他锁。锁状态由内核句柄管理；锁文件本身持久存在，且应被 Git 忽略。
pub struct ProjectLock {
    file: Option<File>,
}

impl ProjectLock {
    pub fn acquire(lock_dir: &Path) -> Result<Self> {
        if !lock_dir.is_dir() {
            return Err(
                WorkspaceError::runtime("Itemark root is not initialised; cannot write")
                    .at(lock_dir),
            );
        }
        let lock_path = lock_dir.join(".itemark.lock");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|error| {
                WorkspaceError::runtime(format!("cannot open project lock file: {error}"))
                    .at(&lock_path)
            })?;

        let deadline = Instant::now() + LOCK_TIMEOUT;
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Self { file: Some(file) }),
                Err(error) if is_contention(&error) => {
                    if Instant::now() >= deadline {
                        return Err(WorkspaceError::runtime(
                            "another Itemark process is writing to this project; retry shortly",
                        )
                        .at(&lock_path));
                    }
                    std::thread::sleep(RETRY_INTERVAL);
                }
                Err(error) => {
                    return Err(WorkspaceError::runtime(format!(
                        "cannot acquire project lock: {error}"
                    ))
                    .at(&lock_path));
                }
            }
        }
    }
}

impl Drop for ProjectLock {
    fn drop(&mut self) {
        self.file.take();
    }
}

fn is_contention(error: &std::fs::TryLockError) -> bool {
    match error {
        std::fs::TryLockError::WouldBlock => true,
        std::fs::TryLockError::Error(error) => {
            error.kind() == std::io::ErrorKind::WouldBlock || error.raw_os_error() == Some(11)
        }
    }
}
