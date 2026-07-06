# WezTeam - AI Agent 团队协作终端

[English Documentation →](README.md)

---

## 这是什么？

基于 WezTerm 的终端模拟器，内置 **异构 AI Agent 团队协作** 能力。

**你不需要学习任何新命令**，像使用普通 Agent 一样对话即可。

**核心特性**：在同一个终端中运行多个**不同**的 Agent（Claude、Codex、Gemini、Qwen、Cursor 等），让它们协同完成复杂任务。

---

## 工作原理

想象你在开一家公司：

- **WezTeam** = 你的办公室（终端）
- **Agent** = 你的员工（Claude、Codex、Gemini、Qwen、Cursor 等）
- **TeamShell** = 公司规章制度（协作协议）
- **tsh** = 内部管理工具（Agent 自己会用）

**WezTeam 的核心：异构 Agent 协作**

你可以在同一个终端中运行多个**不同模型**的 Agent，让它们做**单个 Agent 做不到的事**。

---

### 场景一：多 Agent 委员会评审

**单 Agent 的问题**：自己写代码、自己评审，容易 blind spot。

**WezTeam 的解法**：

```
你: "帮我设计一个用户登录系统"

  ↓ Claude 在标签页 1 产出第一版需求文档

[标签页 2 - Qwen] [标签页 3 - Codex]
        ↓                ↓
    开始评审          开始评审

[标签页 2 - Qwen]
"需求文档 v1，请评审"
→ "安全性不足，密码强度要求缺失，评分 6/10"

[标签页 3 - Codex]
"需求文档 v1，请评审"
→ "性能考虑不周，并发场景未覆盖，评分 5/10"

[标签页 1 - Claude]
→ "收到反馈，我更新 v2"

[标签页 2 - Qwen]
→ "v2 有所改进，但错误处理不够细，评分 7/10"

[标签页 3 - Codex]
→ "架构合理，边界条件还需补充，评分 7/10"

  ↓ [经过几轮讨论后]

[标签页 1 - Claude]
→ "需求文档 v3 已完成，综合评分 8.5/10"
→ 向你汇报最终结果
```

**关键价值**：
- **不同模型，不同视角**：Claude 的架构思维、Qwen 的工程严谨性、Codex 的实践经验
- **互相质疑，互相评审**：不是单方面输出，而是多轮讨论
- **逼近严谨**：单 Agent 容易自满，多 Agent 互相挑战才能出精品
- **过程透明**：你在终端标签页中**实时看到**整个评审过程

---

### 场景二：开发 + 评审 + 测试流水线

**单 Agent 的问题**：写完代码直接给你，质量不可控。

**WezTeam 的解法**：

```
你: "帮我实现登录功能"

  ↓ Claude 在标签页 1 负责开发

Claude → 完成代码 → 写入文件

  ↓ Codex 在标签页 2 负责评审

Codex → 读取代码 → 发现 3 个问题：
  - 缺少输入验证
  - 错误处理不完善
  - 性能可优化

Codex → [标签页 1] 告诉 Claude: "请修复以上问题"
Claude → 修复 → Codex → 二次评审 → 通过

  ↓ Qwen 在标签页 3 负责测试

Qwen → 运行测试 → 发现边界 case 问题
Qwen → [标签页 1] 告诉 Claude: "测试失败，请修复"
Claude → 修复 → Qwen → 重新测试 → 通过

  ↓ Claude 向你汇报

Claude: "登录功能已完成，Codex 评审通过，Qwen 测试通过"
```

**关键价值**：
- **专业分工**：开发、评审、测试各司其职
- **质量闭环**：不是写完就完事，而是经过评审和测试
- **单向流**：避免互相干扰，每个 Agent 专注自己的角色
- **过程透明**：你在三个标签页中**实时看到**开发、评审、测试的完整过程

---

### 两个场景的共同特点

| 特点 | 说明 |
|------|------|
| **异构 Agent** | Claude + Qwen + Codex，不同模型不同优势 |
| **单向协作** | 不是所有 Agent 一起输出，而是有顺序、有角色 |
| **互相评审** | Agent 之间不是单纯执行，而是会质疑、审核、打分 |
| **质量导向** | 不是追求快，而是追求严谨、可靠、可交付 |
| **用户零负担** | 你只需要布置任务，过程完全自动 |

**你不需要知道 `tsh` 命令，不需要管理 Agent，不需要手动协调。**

你只需要说：**"帮我做一个登录功能"**

剩下的事，Agent 们自己会搞定。

---

## 快速开始

### 下载安装

**Windows**：下载 `WezTerm-*-setup.exe` → 双击安装

**macOS**：下载 `WezTerm-*-macos.zip` → 解压拖到 Applications

**Linux**：
- Ubuntu/Debian：`wget ... && tar -xf ... && sudo dpkg -i .`
- Fedora：`wget ... && sudo dnf install ...`
- 通用：下载 AppImage，`chmod +x` 后运行

---

### 配置（只需一次）

```bash
tsh init
```

这会自动把协作协议写入你系统中的 Agent 配置文件。

**如果遇到问题**？在对话中 @Agent 说：**"请阅读 @TeamShellProtocol.md"**，效果一样。

---

### 开始使用

**启动单个 Agent**：

```bash
claude    # Claude
# codex   # Codex
# gemini  # Gemini
# qwen    # Qwen
```

**启动多个不同 Agent**，让它们协同工作：

```bash
# 标签页 1: Claude 负责架构设计
claude

# 标签页 2: Qwen 负责前端开发
qwen

# 标签页 3: Codex 负责后端开发
codex
```

Agent 第一次看到协议后，会**自动完成入职**并向你汇报。

之后就像使用普通 Agent 一样对话：

```
"帮我做一个登录功能"
```

Agent 会自动：
- 理解你的需求
- 使用 TeamShell 协议与其他 Agent 通信、协调
- 向你汇报结果

**你不需要知道任何内部细节。**

---

## 开发者文档

如果你想**从源码构建**或**贡献代码**，请阅读：

📖 **[BUILD.md](BUILD.md)**（英文，独立文档）

---

## 更多资料

- 📖 **[TeamShellProtocol.md](TeamShellProtocol.md)** — 完整协作协议（Agent 阅读）
- 📖 **[AGENTS.md](AGENTS.md)** — Agent 配置指南
- 🎨 **[标签栏配置](docs/config/lua/config/tab_bar_extra_info.md)** — 自定义垂直标签栏
- 📦 **[Releases](https://github.com/wsyb/wezteam/releases)** — 下载最新版本

---

## 关于本项目

基于 [WezTerm](https://github.com/wezterm/wezterm) 的增强分支。

- **上游**：[wezterm/wezterm](https://github.com/wezterm/wezterm)
- **许可证**：[MIT](LICENSE.md)
