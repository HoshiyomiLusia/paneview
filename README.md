# PaneView

PaneView 是一个 Rust 编写的跨平台终端 UI 工具，用于在一个 TUI 中分屏运行本机 shell，并查看基础系统状态。

目标平台：

- macOS
- Linux

第一版是 MVP，不追求完整替代 tmux。

## 功能

- 使用 `ratatui` + `crossterm` 渲染终端 UI。
- 每个 pane 使用 PTY 启动独立 shell。
- 支持纵向分屏，也就是左右分屏。
- 支持横向分屏，也就是上下分屏。
- 支持切换焦点 pane。
- 普通键盘输入会发送到当前焦点 pane。
- `Ctrl+C` 会发送给当前 pane 内的进程，不会退出 PaneView。
- 支持关闭当前 pane。
- 支持显示或隐藏图形化系统信息面板。
- 系统面板用进度条和火花线显示 CPU、内存、磁盘、网络接口、IP、网络速率、系统版本、内核、主机名和运行时间。

## 安装与运行

需要安装 Rust 工具链。

```bash
cargo build
```

安装为当前用户可直接执行的命令：

```bash
cargo install --path .
```

安装后可以直接运行：

```bash
paneview
```

运行：

```bash
cargo run
```

或运行编译后的二进制：

```bash
./target/debug/paneview
```

## 快捷键

| 快捷键 | 功能 |
| --- | --- |
| `Ctrl+Q` | 退出程序 |
| `Ctrl+H` | 聚焦左侧 pane |
| `Ctrl+J` | 聚焦下方 pane |
| `Ctrl+K` | 聚焦上方 pane |
| `Ctrl+L` | 聚焦右侧 pane |
| `Ctrl+\` | 纵向分屏，生成左右 pane |
| `Ctrl+-` | 横向分屏，生成上下 pane |
| `Ctrl+N` | 新建 pane，默认使用纵向分屏 |
| `Ctrl+W` | 关闭当前 pane |
| `Ctrl+S` | 显示或隐藏系统信息面板 |
| `Ctrl+C` | 发送到当前 pane 内进程 |

## 项目结构

```text
src/
  app.rs      程序状态、pane 管理、焦点管理
  input.rs    键盘事件到动作的转换
  layout.rs   分屏布局树和布局测试
  main.rs     终端初始化后的主循环
  pane.rs     PTY、shell 进程、输出缓冲
  system.rs   macOS/Linux 系统信息采集
  tui.rs      ratatui 渲染
```

## 已知限制

- 这是 MVP，pane 内部不是完整终端模拟器，但使用 `vt100` 解析常见 ANSI 输出。
- 当前没有命令面板。每个 pane 默认启动用户 shell，命令在 shell 中输入执行。
- 关闭最后一个 pane 会被拒绝，以避免 UI 没有主工作区。
- 不做 GPU、温度、风扇、抓包、远程主机管理或插件系统。
- 某些系统指标在特定平台不可用时会显示 `N/A`。

## 验证

```bash
cargo fmt
cargo check
cargo test
```
