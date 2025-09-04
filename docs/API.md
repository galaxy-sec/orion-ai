# API使用指南

## 基础示例

```rust
use orion_ai::*;

#[tokio::main]
async fn main() -> AiResult<()> {
    // 1. 初始化注册表
    GlobalFunctionRegistry::initialize()?;

    // 2. 创建执行单元
    let ai = AiExecUnitBuilder::new(load_config()?).build();

    // 3. 执行智能请求
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

## 动态工具注册

Orion AI 支持在运行时动态注册新的函数工具和执行器，这允许第三方 crate 扩展系统功能。

### 基本使用

#### 注册单个函数

```rust
use orion_ai::*;

// 创建函数定义
let custom_function = FunctionDefinition {
    name: "my-custom-tool".to_string(),
    description: "我的自定义工具".to_string(),
    parameters: vec![],
};

// 注册函数
GlobalFunctionRegistry::register_function(custom_function)?;
```

#### 注册执行器

```rust
// 自定义执行器
struct MyExecutor;

#[async_trait::async_trait]
impl FunctionExecutor for MyExecutor {
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        // 实现执行逻辑
        Ok(FunctionResult {
            name: function_call.function.name.clone(),
            result: serde_json::json!({"custom": "result"}),
            error: None,
        })
    }

    fn supported_functions(&self) -> Vec<String> {
        vec!["my-custom-tool".to_string()]
    }

    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
        // 返回函数schema
        if function_name == "my-custom-tool" {
            Some(FunctionDefinition {
                name: "my-custom-tool".to_string(),
                description: "我的自定义工具".to_string(),
                parameters: vec![],
            })
        } else {
            None
        }
    }
}

let executor = Arc::new(MyExecutor);
GlobalFunctionRegistry::register_executor("my-custom-tool".to_string(), executor)?;
```

#### 批量注册工具集

```rust
let functions = vec![
    FunctionDefinition {
        name: "tool-1".to_string(),
        description: "工具1".to_string(),
        parameters: vec![],
    },
    FunctionDefinition {
        name: "tool-2".to_string(),
        description: "工具2".to_string(),
        parameters: vec![],
    },
];

let executor = Arc::new(MyExecutor);
GlobalFunctionRegistry::register_tool_set(functions, executor)?;
```

### 第三方 Crate 扩展示例

```rust
// 在第三方 crate 中
pub fn register_my_extension() -> AiResult<()> {
    let functions = create_my_functions();
    let executor = Arc::new(MyExtensionExecutor::new());

    GlobalFunctionRegistry::register_tool_set(functions, executor)
}
```

### API 参考

#### GlobalFunctionRegistry

- `register_function(function: FunctionDefinition) -> AiResult<()>` - 注册单个函数定义
- `register_executor(function_name: String, executor: Arc<dyn FunctionExecutor>) -> AiResult<()>` - 注册执行器
- `register_tool_set(functions: Vec<FunctionDefinition>, executor: Arc<dyn FunctionExecutor>) -> AiResult<()>` - 批量注册工具集
- `unregister_function(function_name: &str) -> AiResult<()>` - 注销函数

## 环境要求
- Rust 1.75+
- Tokio runtime
- 配置文件: _gal/*

更多详细指南参见 README.md 和 TOOLS.md
