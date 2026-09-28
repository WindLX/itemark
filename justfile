# Worklog 仓库门禁。
#
# 用法：
#   just              跑完整门禁（等价于 just ci）
#   just check        只做编译检查
#   just lint         只做 lint（clippy）
#   just test         只跑测试
#   just --list       查看全部配方

set shell := ["bash", "-euo", "pipefail", "-c"]

# 默认配方：跑完整门禁
default: ci

# 完整门禁：格式 → 编译 → lint → 测试
ci: fmt-check check lint test

# 编译检查，含测试与示例目标
check:
    cargo check --all-targets --locked

# lint：所有警告视为错误
lint:
    cargo clippy --all-targets --locked -- -D warnings

# 运行全部测试
test:
    cargo test --locked

# 格式检查（只检查，不修改文件）
fmt-check:
    cargo fmt --all -- --check

# 按 rustfmt 写回格式
fmt:
    cargo fmt --all

# 删除构建产物
clean:
    cargo clean
