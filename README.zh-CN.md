# WezTeam - AI Agent 团队协作终端

[English Documentation →](README.md)

---

## 这是什么？

基于 WezTerm 的终端模拟器，内置 **异构 AI Agent 团队协作** 能力。

**你不需要学习任何新命令**，像使用普通 Agent 一样对话即可。

---

## 为什么需要它？

### 单 Agent 的局限

你有一个 Claude，它能帮你写代码、改 bug、分析项目。

但当任务变复杂时，单 Agent 有天然的局限：
- **自己评审自己的代码** → 容易 blind spot
- **一个视角分析需求** → 容易遗漏边界 case
- **写完直接给你** → 质量不可控

### WezTeam 的解法

让**不同模型**的 Agent 在同一个终端中协作：
- Claude 写需求 → Qwen + Codex 评审 → 多轮迭代
- Claude 开发 → Codex 评审 → Qwen 测试 → 质量闭环

**单个 Agent 发现 2 个问题，多 Agent 委员会能发现 7 个。**

你不需要知道内部怎么运作，只需要说一句话。

---

## 它适合谁？

| 人群 | 是否适合 | 说明 |
|------|---------|------|
| 想提升代码质量的开发者 | ✅ **非常适合** | 多 Agent 评审 = 更少的 bug |
| 需要严谨需求文档的 PM / 架构师 | ✅ **非常适合** | 多 Agent 委员会 = 更严谨的文档 |
| 日常使用 AI 编程的用户 | ✅ 适合 | 即使只有一个 Agent 也能用 |
| 想研究 AI 协作的研究者 | ✅ **非常适合** | 透明化的协作过程 = 可观察的实验场 |
| 完全没用过 AI 工具的新手 | ⚠️ 可以试试 | 先从单个 Agent 开始 |

**即使你只有一个 Claude，也可以用 WezTeam。**

---

## 工作原理

想象你在开一家公司：

- **WezTeam** = 你的办公室（终端）
- **Agent** = 你的员工（Claude、Codex、Gemini、Qwen 等）
- **TeamShell** = 公司规章制度（协作协议）
- **tsh** = 内部管理工具（Agent 自己会用）

**核心：异构 Agent 协作**

在同一个终端中运行多个**不同模型**的 Agent，让它们做**单个 Agent 做不到的事**。

每个 Agent 在独立的标签页中运行，**你实时看到**整个过程。

---

### 场景一：多 Agent 委员会评审

**单 Agent**：自己写、自己评审 → 发现 2 个问题

**WezTeam**：多个 Agent 互相评审 → 发现 7 个问题

```
你: "帮我设计一个用户登录系统"

  ↓ Claude 在标签页 1 产出第一版需求文档

[标签页 2 - Qwen] [标签页 3 - Codex]
        ↓                ↓
    开始评审          开始评审

[标签页 2 - Qwen]
"需求文档 v1，请评审"
→ "密码使用 MD5 哈希，应改为 bcrypt + salt，评分 6/10"

[标签页 3 - Codex]
"需求文档 v1，请评审"
→ "登录接口缺少速率限制，应加 Redis 防暴力破解，评分 5/10"

[标签页 1 - Claude]
→ "收到，我更新 v2"

[标签页 2 - Qwen]
→ "v2 密码安全已修复，但并发场景存在竞争条件，应加分布式锁，评分 7/10"

[标签页 3 - Codex]
→ "架构合理，但错误码规范未统一，评分 7/10"

  ↓ [经过几轮讨论后]

[标签页 1 - Claude]
→ "需求文档 v3 已完成"
→ 向你汇报：综合评分 8.5/10，共修复 7 处问题
```

**关键价值**：
- **不同模型，不同视角**：Claude 的架构思维 + Qwen 的工程严谨性 + Codex 的实践经验
- **互相质疑，互相评审**：不是单方面输出，而是多轮讨论
- **过程透明**：你在终端标签页中**实时看到**整个评审过程

---

### 场景二：开发 → 评审 → 测试流水线

**单 Agent**：写完代码直接给你 → 你手动发现问题

**WezTeam**：写 → 评审 → 改 → 测试 → 改 → 通过 → 交付

```
你: "帮我实现登录功能"

  ↓ Claude 在标签页 1 负责开发

Claude → 完成代码 → 写入文件

  ↓ Codex 在标签页 2 负责评审

Codex → 读取代码 → 发现 3 个问题
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

Claude: "登录功能已完成，Codex 评审通过，Qwen 测试通过，可直接合并"
```

**关键价值**：
- **专业分工**：开发、评审、测试各司其职
- **质量闭环**：评审 + 测试双保险，不是写完就完事
- **单向流**：每个 Agent 专注自己的角色，不互相干扰
- **过程透明**：三个标签页，实时看到完整的开发和验证过程

---

### 两个场景的共同特点

| 特点 | 说明 |
|------|------|
| **异构 Agent** | Claude + Qwen + Codex，不同模型不同优势 |
| **互相评审** | Agent 之间不是单纯执行，而是会质疑、审核、打分 |
| **多轮迭代** | 不是一次通过，而是逼近严谨 |
| **质量导向** | 追求严谨、可靠、可交付 |
| **过程透明** | 每个标签页实时可见，全程可追溯 |
| **用户零负担** | 你只需要布置任务，过程完全自动 |

**你不需要知道 `tsh` 命令，不需要管理 Agent，不需要手动协调。**

你只需要说：**"帮我做一个登录功能"**

剩下的事，Agent 们自己会搞定。

---

## 快速开始

### 下载安装

**Windows**：下载 `WezTerm-*-setup.exe` → 双击安装

**macOS**：下载 `WezTerm-*-macos.zip` → 解压拖到 `/Applications`

**Linux**：
```bash
# Ubuntu/Debian — 下载 .deb 安装
wget https://github.com/wsyb/wezteam/releases/download/v1.0.0/wezterm-1.0.0.Ubuntu24.04.deb
sudo dpkg -i wezterm-1.0.0.Ubuntu24.04.deb

# Fedora/CentOS — 下载 .rpm 安装
wget https://github.com/wsyb/wezteam/releases/download/v1.0.0/wezterm-1.0.0-1.fedora41.x86_64.rpm
sudo dnf install ./wezterm-1.0.0-1.fedora41.x86_64.rpm

# 通用版 — 下载 AppImage
chmod +x WezTerm-*.AppImage
./WezTerm-*.AppImage
```

---

### 配置（只需一次）

```bash
tsh init
```

这会自动把协作协议写入你系统中的 Agent 配置文件。

**如果遇到问题**？打开 WezTeam，在新标签页启动 Agent 后，直接在对话中说：

```
请阅读 @TeamShellProtocol.md
```

Agent 会读取协议内容，效果与 `tsh init` 相同。

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
