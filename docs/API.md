# API使用指南

## 基础示例

```rust
use orion_ai::*;

#[tokio::main]
async fn main() -> AiResult<()> {
    // 1. 初始化注册表
    GlobalFunctionRegistry::initialize()?;
    
    // 2. 选择工具集
    let tools = vec!["git-status", "fs-ls"];
    
    // 3. 创建执行单元
    let ai = AiExecUnitBuilder::new(load_config()?).build();
    
    // 4. 执行智能请求
    let response = ai.smart_request("检查这个仓库状态").await?;
    println!("AI响应: {}", response);
    Ok(())
}
```

## 高级功能

### 角色定制
预定义角色:
- developer: 开发者模式
- operations: 运维模式
- galactiward: 银河任务模式

### 线程记录
自动记录所有AI交互到本地线程文件，支持历史回顾。

## 环境要求
- Rust 1.75+
- Tokio runtime
- 配置文件: _gal/*

更多详细指南参见 README.md 和 TOOLS.md
