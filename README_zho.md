<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-SUITE-LAUNCHER banner" width="100%">
</p>

# 📖 EH3D-SUITE-LAUNCHER

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | 🇨🇳 <b>中文</b> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### EH3D Fabrication Suite 的中心：查找并启动程序，管理项目文件

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Version-0.0.5-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**诚实检查 - 当前真正能运行的内容：** 一个 Rust 库和一个命令行程序（`eh3d-launcher`），有 119 个在电脑上运行的测试：通过清单找到套件的程序，启动它们并在失败时重新启动，保存最近的项目，并用每个文件的 SHA-256 写入和校验项目文件。窗口（`eh3d-launcher-gui`，用 `gui` 特性构建）已编写，其逻辑已测试，但尚未构建，也没有在屏幕上看过；没有在机器上试过任何东西，也没有用大量真实文件验证过。

---

> **安全状态：** 本程序不控制任何机器，也不是安全系统。它的输出在机器上运行之前必须由人检查；这里的任何内容都没有在真实硬件上运行过。


## 📋 速览

| | |
|---|---|
| 版本 | `0.0.5` |
| 成熟度 | 脚手架 |
| 测试 | 119 |
| 技术栈 | Rust |
| 真实硬件 | 未试过：这里的任何内容都没有在机器上运行过 |

**跳转到：** [概述](#-概述) · [架构](#-架构) · [快速开始](#-快速开始) · [测试](#-测试) · [结构](#-仓库结构) · [相关项目](#-相关项目)

## 🚦 项目状态

每一行是本仓库的一个部分，灯号表示它今天做到了哪一步。内容依据仓库能够展示的证据（测试、构建、运行），而不是计划。

🟢 已完成并验证 · 🟡 可用，有已知限制 · 🟠 部分完成或尚未验证 · 🔴 未完成或不可用

| 领域 | 状态 | 在这里的含义 |
|---|---|---|
| 核心 | 🟡 | 通过清单找到套件的程序，启动它们并在失败时重新启动，保存最近的项目，并用每个文件的 SHA-256 写入和校验项目文件；已在电脑上用正常和损坏的输入测试，未用大量真实文件验证。 |
| 命令行 | 🟡 | `eh3d-launcher` 运行与库相同的函数并打印结果；每个错误都以消息和非零退出码返回。 |
| 测试 | 🟡 | 119 个测试通过（`cargo test`），包含正常情况、边界情况和损坏的输入。 |
| 安全 | 🟡 | 纯安全 Rust（`forbid(unsafe_code)`）：处理收到的文件时不会 panic，错误的值会返回带原因的错误；没有独立审查。 |
| 窗口 | 🟠 | 已编写（egui，`--features gui`，程序 `eh3d-launcher-gui`）：最近的项目、打开或新建项目、用与命令行相同的检查添加和移除其文件，以及套件的程序（每个都有启动按钮）；其逻辑已测试，但尚未构建，也没有在屏幕上看过。 |
| 真实硬件 | 🔴 | 未试过：这里的任何内容都没有在机器上运行过。 |
| 文档 | 🟡 | 八种语言的 README、更新日志，以及由 `docs/` 绘制的两张图。 |

## 🎯 概述

* **注册表**（`src/registry.rs`）：套件的每个仓库在 `eh3d.project.json` 中说明自己是什么；启动器读取它们，带原因跳过损坏的，并告知哪些程序已构建。
* **启动器**（`src/launch.rs`）：启动套件的程序，如果以错误结束，则最多重新启动两次，等待时间每次加倍；正常结束的程序不再处理。
* **项目文件**（`src/project.rs`，`.eh3dlist`）：名称、作业文件（含类型和 SHA-256）和工作零点；读取时拒绝离开项目的路径，并迁移第一版格式。**最近项目**（`src/recent.rs`）保存最近十个。
* **查找与重启：** 程序的可执行文件按其 `Cargo.toml` 中的 `[[bin]]` 名称查找（`eh3d-camdebug`、`eh3d-pnp` 等）；操作员有意结束的程序（信号）或拒绝了参数的程序（退出码 1 或 2）不会重启，只有崩溃才会。
* **窗口**（`src/window.rs`、`src/gui.rs`；`--features gui`，程序 `eh3d-launcher-gui`）：最近的项目、打开或新建项目、用与 `add-layer` 相同的检查添加和移除其文件，以及套件的程序（用按钮带着项目启动每一个）。状态、操作、设置和文字都是有测试的普通代码；绘制层很薄。已编写，尚未在屏幕上看过。

## 🧩 在 EH3D 中的角色

EH3D-SUITE-LAUNCHER 是 EH3D Fabrication Suite 的一个程序。它以库、命令行的形式工作，用 `gui` 特性构建后还是与套件其他程序外观一致的窗口（`eh3d-launcher-gui`）。它读写简单的文件，因此可以单独使用，也可以用在其他程序之间。

## 🏛️ 架构

<p align="center">
  <img src="images/architecture.svg" alt="EH3D-SUITE-LAUNCHER 的架构：谁在使用、包含什么、依赖什么" width="100%">
</p>


本仓库各部分如何配合：谁使用它、它包含什么、它依赖什么。图由 `docs/architecture.json` 绘制并与代码核对。

## ✨ 功能

* **库加命令行：** `eh3d-launcher` 的每个命令都是库中的函数，套件中的其他程序无需启动进程即可调用。
* **损坏输入的测试：** 测试还会输入空文件、乱码、巨大数字和被截断的文件，任何情况都不允许 panic。
* **不猜测：** 没有意义的值是一个说明哪个值、为什么的错误，绝不是只是看起来正确的结果。
* **读取文件的大小上限：** 仓库的清单和最近项目列表在命令行和窗口中，仅当不超过 `launcher.cfg` 中的 `max_file_mib`（默认 16 MiB，范围 1 到 1024）时才会读取，`launcher.cfg` 本身最多 1 MiB；过大的清单与其他损坏的清单一样，以代码 `file_too_large` 和名称被跳过，列表过大时 `eh3d-launcher recent` 以该代码结束，窗口以空列表启动。其他代码是 `file_unreadable` 和 `file_not_text`。
* **没有 unsafe 代码：** 库和程序中都有 `#![forbid(unsafe_code)]`。
* **更安全的列表和项目：** 图层路径不能是 `.`、Windows 不接受的名称（`CON`、`AUX`、`COM1` 等），也不能以点或空格结尾，仅大小写不同的两个图层视为同一个，最近列表保存现有文件的完整路径，同时运行的两个操作（`recent`、`add-layer`）不会丢失对方的修改，因为文件在修改期间被锁定；图层按 64 KiB 的块计算哈希，被信号终止的程序会如实说明而不是当作失败。

## 🔒 安全与防护

| 威胁 | 防御 |
|---|---|
| 损坏、被截断或为空的文件 | 不会 panic：测试在每个位置截断有效输入并输入乱码 |
| 没有意义的数字（负数、非有限值、过大的数） | 以指出该值的错误拒绝；不会被舍入成看似合理的值 |
| 不安全的内存使用 | 带 `forbid(unsafe_code)` 的纯安全 Rust |

**没有防御的内容：** 无错误产生的结果不等于经过验证的结果。这里没有任何内容了解机器的极限，程序也没有经过独立审查。请私下报告漏洞（见 [SECURITY.md](SECURITY.md)）。

## 🔌 接口

库为 `eh3d_suite_launcher`（`use eh3d_suite_launcher::...`），命令行为 `eh3d-launcher`。命令如下；每个命令把结果打印到标准输出，把错误以非零退出码报告到标准错误。没有网络访问。

```text
EH3D-SUITE-LAUNCHER/
├── src/
│   ├── browse.rs                                打开文件窗口的文件列表
│   ├── gui.rs                                   窗口（egui，gui 特性）：绘制状态并传递用户的操作
│   ├── gui_main.rs                              作为独立程序的窗口，在 Windows 上没有控制台
│   ├── lang.rs                                  窗口的文字（八种语言）
│   ├── launch.rs                                逻辑的一部分
│   ├── limit.rs                                 带大小上限的读取，每种拒绝都有代码
│   ├── lib.rs                                   库：每个命令所使用的部分
│   ├── main.rs                                  命令行
│   ├── project.rs                               逻辑的一部分
│   ├── recent.rs                                逻辑的一部分
│   ├── registry.rs                              逻辑的一部分
│   ├── settings.rs                              launcher.cfg 的设置：语言、重启次数和窗口大小
│   ├── shot.rs                                  窗口的截图模式（--shot）：无需屏幕即可读取和检查其参数
│   └── window.rs                                无需屏幕的窗口状态：程序、项目、文件和启动
├── tests/
│   ├── launcher.rs                              库和命令行的测试
│   ├── read_limit.rs                            所读文件大小上限的测试
│   └── shot_sample.rs                           截图窗口中示例的测试
├── examples/
│   ├── workspace/                               套件程序的示例清单，用于在截图窗口中显示
│   └── demo/                                    包含三个文件及其校验和的示例项目列表
├── docs/
│   ├── architecture.json                        架构图数据
│   └── structure.json                           结构图数据
├── tools/                                       CI 和 README 检查及绘图工具
├── images/                                      横幅图以及架构和仓库结构图片
├── .github/                                     CI 与发布工作流、Issue 和 PR 模板
├── eh3d.project.json                            清单：版本、角色、成熟度和构建命令
├── Cargo.toml Cargo.lock                        工作区及其锁定的依赖
├── rust-toolchain.toml                          固定的 Rust 工具链
├── CHANGELOG.md                                 每个版本的变更
├── SECURITY.md SUPPORT.md                       如何报告漏洞及在哪里提问
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   参与贡献、行为准则和许可证
└── launcher.cfg.example                         launcher.cfg 的所有设置及其默认值
```

## ⚙️ 配置

配置文件是启动器文件夹中的 `launcher.cfg`，由 `launch` 和窗口读取：其中的 `workspace = 完整路径` 一行是启动程序所在的文件夹（只启动套件中已知的程序），可选的 `language`、`max_restarts`、`backoff_ms`、`window_width` 和 `window_height` 行设置窗口的语言、崩溃的程序重新启动的次数、等待时间和窗口大小。`launcher.cfg.example` 列出了每个键及其默认值和限制。除此之外，用户的选择通过命令行参数给出（见接口），每个默认值都写在命令的帮助中（`eh3d-launcher help`）。值的限制是代码中的具名常量，并有测试覆盖。

## 🚀 快速开始

需要稳定版 Rust 工具链（1.95 或更新）。

```bash
cargo build --release
cargo test
cargo run --release -- list --workspace ..
# 窗口（需要 gui 特性）
cargo build --release --features gui
cargo run --release --features gui --bin eh3d-launcher-gui
cargo run --release --features gui --bin eh3d-launcher-gui -- --shot screen.png --open examples/workspace --open examples/demo/demo.eh3dlist   # a picture of the window
```

要在不手动打开窗口的情况下截取窗口图像，请使用 `--shot FILE.png` 运行（还可用 `--size WxH`, `--theme forge|dark|light`, `--wait N`, `--lang CODE`, `--open FILE`, `--replace`）：它按该尺寸打开，保存 PNG，不保留任何设置并自动关闭（写入文件成功时退出码为 0，否则为 1，参数错误时为 2）。

## 🧪 测试

测试套件有 119 个测试（`cargo test`，CI 在 Windows 和 Linux 上运行）。它们检查正常情况、边界情况和损坏的输入：空文件、乱码和被截断的文件绝不会 panic。CI 还运行 `cargo fmt --check`、把警告当作错误的 `cargo clippy` （也包括 `--features gui`） 以及 `python -I tools/ci_validate.py`。

## 🏗️ 构建与发布

版本号位于 `Cargo.toml` 和 `eh3d.project.json`；每个版本在 [CHANGELOG.md](CHANGELOG.md) 中都有条目。`build.bat` / `build.sh` 构建程序，`test.bat` / `test.sh` 运行测试，`run.bat` / `run.sh` 用示例运行。版本发布在 GitHub 上。

### 下载

每个版本都包含适用于 Windows（64 位和 32 位）、Linux（64 位）和 macOS（Apple silicon 和 Intel）的命令行程序和窗口，文件名为 `<名称>-<版本>-<系统>.zip`（Windows）或 `.tar.gz`，内含两个程序、许可证、readme、图标和 `launcher.cfg.example`。文件由 GitHub Actions 构建并列在 `SHA256SUMS` 中。要校验下载文件，请把它的哈希与旁边的 `.sha256` 文件比较（`sha256sum -c`）。

## 📂 仓库结构

<p align="center">
  <img src="images/repository-structure.svg" alt="EH3D-SUITE-LAUNCHER 的仓库结构" width="100%">
</p>

<details>
<summary>目录树的文本版</summary>

```text
EH3D-SUITE-LAUNCHER/
├── src/
│   ├── browse.rs                                打开文件窗口的文件列表
│   ├── gui.rs                                   窗口（egui，gui 特性）：绘制状态并传递用户的操作
│   ├── gui_main.rs                              作为独立程序的窗口，在 Windows 上没有控制台
│   ├── lang.rs                                  窗口的文字（八种语言）
│   ├── launch.rs                                逻辑的一部分
│   ├── lib.rs                                   库：每个命令所使用的部分
│   ├── main.rs                                  命令行
│   ├── project.rs                               逻辑的一部分
│   ├── recent.rs                                逻辑的一部分
│   ├── registry.rs                              逻辑的一部分
│   ├── settings.rs                              launcher.cfg 的设置：语言、重启次数和窗口大小
│   ├── shot.rs                                  窗口的截图模式（--shot）：无需屏幕即可读取和检查其参数
│   └── window.rs                                无需屏幕的窗口状态：程序、项目、文件和启动
├── tests/
│   ├── launcher.rs                              库和命令行的测试
│   └── shot_sample.rs                           截图窗口中示例的测试
├── examples/
│   ├── workspace/                               套件程序的示例清单，用于在截图窗口中显示
│   └── demo/                                    包含三个文件及其校验和的示例项目列表
├── docs/
│   ├── architecture.json                        架构图数据
│   └── structure.json                           结构图数据
├── tools/                                       CI 和 README 检查及绘图工具
├── images/                                      横幅图以及架构和仓库结构图片
├── .github/                                     CI 与发布工作流、Issue 和 PR 模板
├── eh3d.project.json                            清单：版本、角色、成熟度和构建命令
├── Cargo.toml Cargo.lock                        工作区及其锁定的依赖
├── rust-toolchain.toml                          固定的 Rust 工具链
├── CHANGELOG.md                                 每个版本的变更
├── SECURITY.md SUPPORT.md                       如何报告漏洞及在哪里提问
├── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   参与贡献、行为准则和许可证
└── launcher.cfg.example                         launcher.cfg 的所有设置及其默认值
```

</details>

## 🗺️ 路线图与已知限制

**目前已知的限制**

* 窗口已编写且其逻辑已测试，但尚未构建，也没有在屏幕上看过。
* 没有在机器上试过，也没有用大量真实文件验证过，也没有独立审查。
* 还不能发现网络上的机器，不负责保持套件为最新（由 EH3D-UPDATER 负责），`.eh3dlist` 是指向文件的 JSON 文件，不是包含这些文件的压缩包。

**下一步**

* 发现网络上车间的机器，并在窗口中调用 EH3D-UPDATER。
* 在每个系统的屏幕上查看窗口，并调整发现的问题。
* 用真实文件和套件的其他程序进行试用。

## 🌍 语言

README 和文本提供英语、西班牙语、法语、意大利语、德语、汉语、日语和欧洲葡萄牙语版本。

## 🔗 相关项目

* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): 项目容器和套件共用类型
* [EH3D-UPDATER](https://github.com/JuanenRac/EH3D-UPDATER): 套件更新程序
* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): 钻孔程序
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): 轮廓铣削程序
* [EH3D-BUNGARD-GUI](https://github.com/JuanenRac/EH3D-BUNGARD-GUI): Bungard 机器的窗口
* [EH3D-PHOENIX-CNC](https://github.com/JuanenRac/EH3D-PHOENIX-CNC): 机器控制程序

## 📚 文档

* [更新日志](CHANGELOG.md)
* [架构与结构数据](docs/architecture.json)：绘制两张图所用的数据。

## 🤝 参与贡献与社区

* 提交拉取请求前请运行 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 和 `python -I tools/ci_validate.py`：必须全部通过。见 [CONTRIBUTING.md](CONTRIBUTING.md) 和[行为准则](CODE_OF_CONDUCT.md)。
* README 有八种语言：修改文字要在八个文件中同时进行，两张图由 `tools/make_architecture.py` 和 `tools/make_structure.py` 根据 `docs/architecture.json` 和 `docs/structure.json` 绘制。
* 请私下报告漏洞：[SECURITY.md](SECURITY.md)。问题、想法和报告：electrohobby3d@gmail.com（[SUPPORT.md](SUPPORT.md)）。

## 👤 作者

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 许可证

软件：GPL-3.0-or-later。见 [LICENSE](LICENSE)。套件的硬件设计使用 CERN OHL-S。
