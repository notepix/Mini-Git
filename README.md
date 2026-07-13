# Mini-Git 🦀

基于 Rust 实现的纯本地轻量级 Git 核心引擎。本项目剥离了复杂的网络同步功能（Push/Pull），专注于在本地重现 Git 最底层的**内容寻址文件系统 (Content-addressable storage)** 原理。

## 📌 项目背景与目标
Git 是开发者日常离不开的工具，但其底层对象模型（Blob, Tree, Commit）对许多人来说依然是个黑盒。本项目旨在：
1. 深入理解 Git 底层的有向无环图 (DAG) 数据结构。
2. 利用 Rust 的强类型系统和所有权模型，重构 C 语言原版 Git 中复杂的指针操作。
3. 提供一个功能完整、无第三方网络依赖的本地版本控制闭环。

## 🚀 核心功能 (Commands)
- `minigit init`：初始化仓库，生成 `.minigit` 隐藏目录结构。
- `minigit hash-object <file>`：计算文件 SHA-1 哈希，并将其 Zlib 压缩后写入对象库。
- `minigit add <file>`：将文件写入暂存区（Index）。
- `minigit commit -m "<msg>"`：生成 Tree 对象和 Commit 对象，构建版本快照。
- `minigit log`：解析 Commit 历史，展示优雅的提交时间线。

## 🏗️ 系统架构与模块划分
项目采用标准 Rust 模块化设计：
- `objects.rs`: 定义统一的 `GitObject` 特征，利用 Rust Enum 和 Struct 精确表达 Blob, Tree, Commit 对象，告别内存越界。
- `storage.rs`: 底层存储引擎，封装 `std::fs` 文件读写、`sha1` 哈希计算与 `flate2` 的 Zlib 编解码。
- `commands.rs`: 业务逻辑层，处理暂存区更新与提交历史树的构建。

## 💡 与原生 Git 开源项目的对比与改进 (参考声明)
本项目核心思想参考了 Linus Torvalds 开发的 [Git](https://github.com/git/git) 原理。在此基础上做出了适合教学与 Rust 语言特性的改进：
1. **更安全的错误冒泡**：原生 C Git 常使用全局状态和直接退出（exit），本项目全面采用 `anyhow::Result` 进行错误传递，保证了文件 I/O 失败时能给出清晰的上下文（Context）。
2. **简化的 Index 结构**：原生 Git 的 Index（暂存区）是极其复杂的二进制结构。为了提升可读性，本项目实现了纯文本格式的 Index 设计，在不影响 Tree 对象生成的前提下，大幅降低了理解门槛。

## 🛠️ 构建与运行
```bash
cargo build --release
./target/release/minigit init
echo "Hello Rust" > test.txt
./target/release/minigit add test.txt
./target/release/minigit commit -m "First commit"
./target/release/minigit log
