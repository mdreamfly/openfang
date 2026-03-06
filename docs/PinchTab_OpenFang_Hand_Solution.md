# OpenFang PinchTab Web Automation Hand 建设方案

## 1. 方案背景与目标

根据最新的 [PinchTab API 架构](https://pinchtab.com/docs/api-structure/) (Tab-scoped) 以及 OpenFang `Hands_cookbook.md` 的规范，我们计划构建一个名为 **"PinchTab Web Automation Hand"** (PinchTab 自动化抓取助手) 的 Hands 插件。

这个 Hand 的核心功能是帮助用户通过自然语言指令，通过 PinchTab 本地 REST API 自主开启浏览器实例、创建标签页、交互（点击、滚动）、抓取文本及 Snapshot 数据，并输出结构化汇总报告。得益于 PinchTab 全新的 **Tab-scoped API 架构**，对 Agent 发出的指令路径大幅简化，极大地降低了大模型的路径拼装错误率（幻觉）。

---

## 2. 目录结构设计

为了对接 OpenFang 的架构，我们需要在 OpenFang 的目录 `crates/openfang-hands/bundled/` 下创建一个名为 `pinchtab_scraper` 的目录：

```text
crates/openfang-hands/bundled/pinchtab_scraper/
├── HAND.toml    # 声明式主配置文件（包含 Meta 信息、Settings、System Prompt 以及 Dashboard）
└── SKILL.md     # 补充知识库文件（作为 PromptOnly 喂给模型 PinchTab 最新 API 的用法）
```

---

## 3. 核心文件实现代码

### 3.1 编写 `HAND.toml`

请在对应路径创建 `HAND.toml`，这是驱动大模型进行阶段性拆分操作的核心。

```toml
id = "pinchtab_scraper"
name = "PinchTab Web Automation Hand"
description = "根据 PinchTab 最新 Tab-Scoped API 构建，用以驱动本地浏览器自动抓取与分析数据。"
category = "productivity"
icon = "🕸️"

# 我们赋予 Agent 需要的文件读写和状态记忆能力，以及最为核心的 shell_exec （调用 curl）
tools = ["shell_exec", "file_write", "memory_store", "memory_recall"]

# 对外暴露给用户填写的自动化任务目标
[[settings]]
key = "target_task"
label = "Automation Task"
description = "请输入要进行的网页自动化操作或抓取目标（网址及意图）"
setting_type = "string"
default = ""

[agent]
name = "pinchtab-scraper"
description = "PinchTab AI Controller"
module = "builtin:chat"
provider = "default"
model = "default"
max_tokens = 8192
temperature = 0.2     
max_iterations = 30

system_prompt = """You are PinchTab Web Automation Hand. Your goal is to autonomously control a local browser via the PinchTab HTTP API to complete the user's task.
Task Objective from User: {{settings.target_task}}

## CRITICAL RULES
- ALWAYS use `shell_exec` with `curl` to interact with PinchTab at `http://localhost:9867`.
- **IMPORTANT**: PinchTab uses a Tab-scoped API design. Do NOT use `/instances/{id}/navigate` or `/instances/{id}/tabs/open`. Use `/tabs/new` and `/tabs/{tab_id}/*`.
- Use `memory_store` to save IDs (instanceId, tabId) to persist state across phases.

## Phase 1 - Environment & Tab Initialization
1. Send `GET /instances` to check for an available browser instance. 
2. If none, send `POST /instances/launch` with `{"mode": "headed"}` to create one, get `instanceId`.
3. Create a tab by sending `POST /tabs/new` with payload `{"instanceId": "<YOUR_INSTANCE_ID>"}`.
4. From the response, save the `tab_id` into `memory_store`.

## Phase 2 - Navigation & Action
1. Send `POST /tabs/{tab_id}/navigate` with `{"url": "TARGET_URL"}`.
2. If interaction is needed, use `POST /tabs/{tab_id}/action` (e.g. `{"kind": "click", "ref": "e5"}`). Get the reference `ref` by calling `GET /tabs/{tab_id}/snapshot?interactive&compact` first.
3. Get the full page text via `GET /tabs/{tab_id}/text`. Wait using a sleep command if the network is slow.

## Phase 3 - Reporting
1. Extract the data mapping to the user's intent.
2. Format a comprehensive summary into a Markdown document.
3. Use `file_write` to save it locally.
4. Call `memory_store` to update dashboard metric: `pinchtab_pages_processed`.
"""

# 定义前端展示给用户的进度卡片
[dashboard]
[[dashboard.metrics]]
label = "Pages Processed"
memory_key = "pinchtab_pages_processed"
format = "number"
```

---

### 3.2 编写 `SKILL.md`

在同级目录下建立 `SKILL.md`，它将通过知识库扩展的形式告诉大模型，全新的 PinchTab 接口长什么样。这样能实现配置与底层 Prompt 的解耦。

```markdown
---
name: pinchtab-api-skill
version: "1.1.0"
description: "Expert knowledge for PinchTab NEW Tab-scoped REST API usages via curl."
runtime: prompt_only
---

# PinchTab NEW API Usage Guide (Tab-scoped)

When controlling the browser, use `shell_exec` with `curl` against `http://localhost:9867`.
**WARNING**: The old `/instances/<id>/action` and `/instances/<id>/navigate` paths are DEPRECATED. Always use `/tabs/...`.

### 1. Instance Management
- **List instances**: `curl -s http://localhost:9867/instances`
- **Launch instance**: 
  ```bash
  curl -s -X POST http://localhost:9867/instances/launch \
  -H "Content-Type: application/json" -d '{"mode": "headed"}'
  ```

### 2. Tab Management (NEW)
- **Create New Tab**: 
  ```bash
  curl -s -X POST http://localhost:9867/tabs/new \
  -H "Content-Type: application/json" -d '{"instanceId": "<INSTANCE_ID>"}'
  ```
  *(Saves response `id` as the Tab ID)*
- **List All Tabs**: `curl -s http://localhost:9867/tabs`
- **List Tabs in Instance**: `curl -s http://localhost:9867/tabs?instanceId=<INSTANCE_ID>`

### 3. Tab Operations (Navigate, Action, Extract)
- **Navigate Tab**:
  ```bash
  curl -s -X POST http://localhost:9867/tabs/<TAB_ID>/navigate \
  -H "Content-Type: application/json" -d '{"url": "https://example.com"}'
  ```
- **Get Interactive HTML Snapshot** (Best way to find interactive elements):
  `curl -s "http://localhost:9867/tabs/<TAB_ID>/snapshot?interactive&compact"`
  *Response example:* `{"elements": [{"ref": "e1", "tag": "button", "text": "Click me"}]}`
- **Click Element by Ref** (Get Ref from snapshot):
  ```bash
  curl -s -X POST http://localhost:9867/tabs/<TAB_ID>/action \
  -H "Content-Type: application/json" -d '{"kind": "click", "ref": "e1"}'
  ```
- **Get Page Text Data**: `curl -s http://localhost:9867/tabs/<TAB_ID>/text`
- **Take Screenshot**: `curl -s http://localhost:9867/tabs/<TAB_ID>/screenshot --output screen.png`
```

---

## 4. 后续接入指南

将这三个文件结构搭建好后，进入你的 OpenFang 源码对应的 Registry 中挂载即可：

打开 OpenFang 代码环境下的 `crates/openfang-hands/src/bundled.rs` 文件，追加 Hand 定义：

```rust
pub fn bundled_hands() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        // ... (其他的已存在的 Hands)
        (
            "pinchtab_scraper",
            include_str!("../bundled/pinchtab_scraper/HAND.toml"),
            include_str!("../bundled/pinchtab_scraper/SKILL.md"),
        ),
    ]
}
```

重新编译 OpenFang 主程序后，用户便可在 Dashboard 界面上从对应的卡片创建该自动抓取助手了。该助手在运行过程中会自我驱动完成对 `localhost:9867` 下拉取信息的要求，并自动整理生成 Markdown。
