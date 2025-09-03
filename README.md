# 🚀 Orion AI - Rust AI调用引擎

[![CI](https://github.com/galaxy-sec/orion-ai/workflows/CI/badge.svg)](https://github.com/galaxy-sec/orion-ai/actions)
[![Coverage Status](https://codecov.io/gh/galaxy-sec/orion-ai/branch/main/graph/badge.svg)](https://codecov.io/gh/galaxy-sec/orion-ai)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://rust-lang.org)


## 🌟 核心特性

### 🔧 智能函数调用
- **完整的 Git 工作流**: `status`, `add`, `commit`, `push`, `pull`, `diff`, `log`
- **文件系统操作**: 安全的 `ls`, `cat`, `find`, `cd`, `pwd`
- **系统信息**: `uname`, `ps`, `df`, `ping` 一键获取
- **智能参数解析**: JSON Schema 驱动的参数验证

### 🎯 多 AI 提供商支持
- **DeepSeek Chat/V3** - 代码开发专用模型
- **OpenAI GPT-4** - 通用智能对话
- **智谱 GLM** - 中文优化，本土模型
- **月之暗面 Kimi** - 超长上下文，中文专家

### 📋 专业角色系统
- **Developer** - 代码分析、调试、重构
- **Operations** - 系统运维、监控、部署
- **Galactiward** - Galaxy任务规划与执行

### 🔄 线程记录系统(TODO)
- **完整会话追踪** - 自动记录所有 AI 交互
- **时间线模式** - 按时间查看历史对话
- **可追溯性** - 每个操作都有上下文记录
- **可重放性** - 支持任务重放和复盘


## 🚀 快速开始

### 1️⃣ 安装

```bash
cargo add orion-ai
```

或直接克隆源码：

```bash
git clone https://github.com/galaxy-sec/orion-ai.git
cd orion-ai
cargo build --release
```

### 2️⃣ 基础配置

创建配置文件 `_gal/ai.yml`:

```yaml
providers:
  deepseek:
    enabled: true
    priority: 1
    api_key: $DEEPSEEK_API_KEY

  openai:
    enabled: true
    priority: 3
    api_key: $OPENAI_API_KEY

  glm:
    enabled: true
    priority: 2

  kimi:
    enabled: true
    priority: 4
```

创建角色配置 `_gal/ai-roles.yml`:

```yaml
roles:
  developer:
    description: "专业开发者助手"
    default_model: deepseek-chat
    rules_per_role: roles/developer/

  operations:
    description: "系统运维专家"
    default_model: glm-4.5
    rules_per_role: roles/operations/
```

### 3️⃣ 30秒上手

```rust
use orion_ai::*;

#[tokio::main]
async fn main() -> AiResult<()> {
    // 1. 初始化系统
    GlobalFunctionRegistry::initialize()?;

    // 2. 构建智能执行单元
    let ai = AiExecUnitBuilder::new(load_config()?)
        .with_role("developer")
        .with_tools(vec![
            "git-status".to_string(),
            "fs-ls".to_string(),
            "fs-cat".to_string()
        ])
        .build();

    // 3. 智能交互 - AI会自动理解你的意图并调用相应工具
    let result = ai
        .smart_request("查看这个git仓库的所有更改并推荐提交消息")
        .await?;

    println!("🎯 AI分析结果：\n{}", result);
    Ok(())
}
```

## 🎯 高级功能示例

### 智能Git工作流

```rust
// AI智能识别提交内容并生成合适消息
let response = ai.smart_request(
    "检查这个仓库的变更，生成一个符合规范的提交消息"
).await?;
```

### 文件智能分析

```rust
// AI会探索目录结构并给出分析
let response = ai.smart_request(
    "分析当前项目结构，找出关键的配置文件"
).await?;
```
