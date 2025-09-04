# 函数工具文档

**Git操作:**
- git-status   - 仓库状态查询
- git-add      - 添加文件到暂存区
- git-commit   - 提交更改
- git-push     - 推送提交
- git-pull     - 拉取更新
- git-diff     - 查看差异
- git-log      - 查看提交历史

**文件系统:**
- fs-ls   - 列出目录
- fs-cat  - 查看文件内容
- fs-find - 搜索文件
- fs-pwd  - 当前目录
- fs-cd   - 更改目录

**系统信息:**
- sys-uname - 系统信息
- sys-ps    - 进程列表
- sys-df    - 磁盘空间
- net-ping  - 网络连通性测试

## 创建自定义工具

Orion AI 支持创建自定义工具来扩展系统功能。以下是完整的开发指南。

### 实现步骤

#### 1. 定义函数 Schema

首先定义你的函数接口：

```rust
use orion_ai::*;

let function_definition = FunctionDefinition {
    name: "my-custom-tool".to_string(),
    description: "自定义工具描述".to_string(),
    parameters: vec![
        FunctionParameter {
            name: "input_text".to_string(),
            description: "输入文本".to_string(),
            r#type: "string".to_string(),
            required: true,
        },
        FunctionParameter {
            name: "optional_param".to_string(),
            description: "可选参数".to_string(),
            r#type: "number".to_string(),
            required: false,
        },
    ],
};
```

#### 2. 实现执行器

实现 `FunctionExecutor` trait：

```rust
struct MyCustomToolExecutor;

#[async_trait::async_trait]
impl FunctionExecutor for MyCustomToolExecutor {
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        // 解析参数
        let args: serde_json::Map<String, serde_json::Value> = 
            serde_json::from_str(&function_call.function.arguments)
                .map_err(|e| OrionAiReason::from_logic(format!("参数解析失败: {}", e)).to_err())?;

        // 获取必需参数
        let input_text = args.get("input_text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OrionAiReason::from_logic("缺少 input_text 参数").to_err())?;

        // 获取可选参数
        let optional_param = args.get("optional_param")
            .and_then(|v| v.as_u64())
            .unwrap_or(42);

        // 执行自定义逻辑
        let result_text = format!("处理结果: {} (参数: {})", input_text, optional_param);

        // 返回结果
        Ok(FunctionResult {
            name: function_call.function.name.clone(),
            result: serde_json::json!({
                "processed_text": result_text,
                "status": "success"
            }),
            error: None,
        })
    }

    fn supported_functions(&self) -> Vec<String> {
        vec!["my-custom-tool".to_string()]
    }

    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
        if function_name == "my-custom-tool" {
            Some(FunctionDefinition {
                name: "my-custom-tool".to_string(),
                description: "自定义工具描述".to_string(),
                parameters: vec![
                    FunctionParameter {
                        name: "input_text".to_string(),
                        description: "输入文本".to_string(),
                        r#type: "string".to_string(),
                        required: true,
                    },
                    FunctionParameter {
                        name: "optional_param".to_string(),
                        description: "可选参数".to_string(),
                        r#type: "number".to_string(),
                        required: false,
                    },
                ],
            })
        } else {
            None
        }
    }
}
```

#### 3. 注册工具

将函数定义和执行器注册到系统中：

```rust
#[tokio::main]
async fn main() -> AiResult<()> {
    // 初始化系统
    GlobalFunctionRegistry::initialize()?;

    // 注册自定义工具
    let function_definition = /* ... */;
    let executor = Arc::new(MyCustomToolExecutor);

    // 方式1：分别注册
    GlobalFunctionRegistry::register_function(function_definition)?;
    GlobalFunctionRegistry::register_executor("my-custom-tool".to_string(), executor)?;

    // 方式2：批量注册（推荐）
    // GlobalFunctionRegistry::register_tool_set(vec![function_definition], executor)?;

    Ok(())
}
```

### 最佳实践

#### 1. 错误处理
- 使用详细的错误信息
- 验证所有必需参数
- 处理边界情况

#### 2. 参数验证
```rust
async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
    let args: serde_json::Map<String, serde_json::Value> = parse_arguments(&function_call.function.arguments)?;
    
    // 验证必需参数
    let text = validate_required_param(&args, "input_text", "string")?;
    let count = validate_optional_param(&args, "count", "number", 10)?;
    
    // 业务逻辑验证
    if text.len() > 1000 {
        return Err(OrionAiReason::from_logic("输入文本过长，最大1000字符").to_err());
    }
    
    // ...
}
```

#### 3. 性能考虑
- 避免阻塞操作
- 使用异步I/O
- 合理设置超时

#### 4. 安全考虑
- 验证所有输入参数
- 避免命令注入
- 限制资源使用

### 示例项目

完整的自定义工具示例请参考 `examples/custom-tools/` 目录。
