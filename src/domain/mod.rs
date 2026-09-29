//! 项目领域词汇与不含 I/O 的取值类型，供记录、查询与命令层共用。

pub mod front_matter;
pub mod id;
pub mod scalar;
pub mod section;

pub use id::{ID_PREFIX, LEGACY_ID_PREFIX, Lifecycle, format_id, id_number};
pub use scalar::Scalar;

/// 记录文件中的一个头部字段。
pub type Field = (String, Scalar);
