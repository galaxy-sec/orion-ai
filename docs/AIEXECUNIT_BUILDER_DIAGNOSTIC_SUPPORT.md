# AiExecUnitBuilder 诊断功能支持文档

## 概述

本文档记录了为 AiExecUnitBuilder 添加诊断配置支持的工作成果，包括实现的功能、修改的文件以及相关的技术细节。

## 背景与目标

为了支持系统诊断功能，我们需要扩展 AiExecUnitBuilder，使其能够构建带有诊断配置的 AiExecUnit 实例。这包括：

1. 在 AiExecUnitBuilder 中添加诊断配置字段
2. 提供设置诊断配置的方法
3. 更新构建逻辑以支持诊断配置
4. 实现相关的错误处理机制

## 实现的功能

### 1. AiExecUnitBuilder 诊断配置支持

#### 新增字段
```rust
#[derive(Clone, Debug, Default)]
pub struct AiExecUnitBuilder {
    // ... 现有字段 ...
    diagnostic_config: Option<DiagnosticConfig>,
}
```

#### 新增方法
- `with_diagnostic_config(mut self, config: DiagnosticConfig) -> Self` - 设置诊断配置
- `with_diagnostic_depth(mut self, depth: DiagnosticDepth) -> Self` - 设置诊断深度级别

#### 构建逻辑更新
- 更新了 `build()` 和 `build_ignoring_tool_errors()` 方法，支持诊断配置
- 根据是否存在诊断配置，调用不同的 AiExecUnit 构造函数

### 2. AiExecUnit 诊断功能扩展

#### 新增字段
```rust
#[derive(Getters, MutGetters, Setters, WithSetters)]
pub struct AiExecUnit {
    // ... 现有字段 ...
    diagnostic_config: Option<DiagnosticConfig>,
}
```

#### 新增构造函数
- `new_with_diagnostic_config(client, role, registry, diagnostic_config)` - 创建带诊断配置的执行单元

#### 新增诊断执行方法
- `execute_diagnosis(depth: DiagnosticDepth) -> AiResult<DiagnosticReport>` - 执行指定深度的诊断
- `execute_diagnosis_with_config(config: DiagnosticConfig) -> AiResult<DiagnosticReport>` - 使用自定义配置执行诊断
- `quick_health_check() -> AiResult<DiagnosticReport>` - 执行快速健康检查
- `standard_diagnosis() -> AiResult<DiagnosticReport>` - 执行标准诊断
- `deep_analysis() -> AiResult<DiagnosticReport>` - 执行深度分析

### 3. 错误处理机制

#### 新增错误类型
在 `AiErrReason` 枚举中添加了 `DiagnosisError(String)` 变体，用于表示诊断过程中的错误。

#### 错误转换方法
为 `OrionAiReason` 添加了 `from_diagnosis(msg: String) -> Self` 方法，用于将诊断错误转换为系统错误。

### 4. 诊断报告格式化

#### 新增方法
为 `DiagnosticReport` 添加了 `formatted_report() -> String` 方法，生成用户友好的诊断报告。

#### 严重程度本地化
为 `IssueSeverity` 添加了 `to_chinese() -> &'static str` 和 `severity_to_chinese() -> &'static str` 方法，支持中文严重程度描述。

## 修改的文件

### 1. src/exec_unit/builder.rs

**主要修改**：
- 添加 `diagnostic_config: Option<DiagnosticConfig>` 字段
- 在 `new()` 方法中初始化 `diagnostic_config: None`
- 添加 `with_diagnostic_config()` 和 `with_diagnostic_depth()` 方法
- 更新 `build()` 和 `build_ignoring_tool_errors()` 方法，支持诊断配置
- 添加相关测试用例

**代码位置**：
- 字段定义：第 12 行
- 初始化：第 27 行
- 新增方法：第 95-115 行
- 构建逻辑更新：第 150-170 行

### 2. src/exec_unit/unit.rs

**主要修改**：
- 添加 `diagnostic_config: Option<DiagnosticConfig>` 字段
- 添加 `new_with_diagnostic_config()` 构造函数
- 添加诊断执行方法：`execute_diagnosis()`, `execute_diagnosis_with_config()`, `quick_health_check()`, `standard_diagnosis()`, `deep_analysis()`
- 添加相关测试用例

**代码位置**：
- 字段定义：第 16 行
- 构造函数：第 39-48 行
- 诊断执行方法：第 88-126 行

### 3. src/error.rs

**主要修改**：
- 在 `AiErrReason` 枚举中添加 `DiagnosisError(String)` 变体
- 为 `OrionAiReason` 添加 `from_diagnosis()` 方法

**代码位置**：
- 错误类型：第 38 行
- 转换方法：第 74-77 行

### 4. src/types/diagnosis.rs

**主要修改**：
- 为 `DiagnosticReport` 添加 `formatted_report()` 方法
- 为 `IssueSeverity` 添加 `to_chinese()` 和 `severity_to_chinese()` 方法
- 添加相关测试用例

**代码位置**：
- 格式化方法：第 608-678 行
- 中文转换方法：第 680-692 行

## 使用示例

### 1. 使用诊断深度配置

```rust
use orion_ai::*;
use orion_variate::vars::EnvDict;

#[tokio::main]
async fn main() -> AiResult<()> {
    let dict = EnvDict::new();
    
    // 创建带有标准诊断深度的执行单元
    let exec_unit = AiExecUnitBuilder::new(dict)
        .with_role("developer")
        .with_diagnostic_depth(DiagnosticDepth::Standard)
        .build()?;
    
    // 执行标准诊断
    let report = exec_unit.standard_diagnosis().await?;
    println!("{}", report.formatted_report());
    
    Ok(())
}
```

### 2. 使用自定义诊断配置

```rust
use orion_ai::*;
use orion_variate::vars::EnvDict;

#[tokio::main]
async fn main() -> AiResult<()> {
    let dict = EnvDict::new();
    
    // 创建自定义诊断配置
    let config = DiagnosticConfig {
        check_basic_info: true,
        check_processes: true,
        check_io_performance: true,
        check_network: true,
        timeout_seconds: 20,
        sampling_interval: 2,
        sampling_count: 5,
    };
    
    // 创建带有自定义诊断配置的执行单元
    let exec_unit = AiExecUnitBuilder::new(dict)
        .with_role("developer")
        .with_diagnostic_config(config)
        .build()?;
    
    // 执行自定义诊断
    let report = exec_unit.execute_diagnosis_with_config(config).await?;
    println!("{}", report.formatted_report());
    
    Ok(())
}
```

### 3. 执行不同级别的诊断

```rust
use orion_ai::*;
use orion_variate::vars::EnvDict;

#[tokio::main]
async fn main() -> AiResult<()> {
    let dict = EnvDict::new();
    let exec_unit = AiExecUnitBuilder::new(dict)
        .with_role("developer")
        .with_diagnostic_depth(DiagnosticDepth::Advanced)
        .build()?;
    
    // 快速健康检查
    let quick_report = exec_unit.quick_health_check().await?;
    println!("快速检查结果:\n{}", quick_report.formatted_report());
    
    // 标准诊断
    let standard_report = exec_unit.standard_diagnosis().await?;
    println!("标准诊断结果:\n{}", standard_report.formatted_report());
    
    // 深度分析
    let deep_report = exec_unit.deep_analysis().await?;
    println!("深度分析结果:\n{}", deep_report.formatted_report());
    
    Ok(())
}
```

## 测试覆盖

### 1. 单元测试

- **AiExecUnitBuilder 测试**：
  - `test_builder_creation` - 验证构建器创建和字段初始化
  - `test_builder_with_diagnostic_config` - 验证诊断配置设置
  - `test_builder_with_diagnostic_depth` - 验证诊断深度设置
  - `test_builder_clone` - 验证构建器克隆功能
  - `test_builder_debug` - 验证构建器调试输出

- **AiExecUnit 测试**：
  - `test_exec_unit_creation` - 验证执行单元创建
  - `test_exec_unit_with_diagnostic_config` - 验证带诊断配置的执行单元
  - `test_diagnosis_methods` - 验证诊断执行方法

- **DiagnosticReport 测试**：
  - `test_diagnostic_report_formatted_report` - 验证诊断报告格式化
  - `test_issue_severity_to_chinese` - 验证严重程度中文转换

### 2. 集成测试

- **构建和诊断流程测试**：
  - `test_build_with_example_config` - 使用示例配置构建和诊断
  - `test_build_with_diagnostic_config` - 带诊断配置的构建和诊断
  - `test_build_ignoring_tool_errors` - 忽略工具错误的构建和诊断
  - `test_build_ignoring_tool_errors_with_diagnostic_config` - 带诊断配置的忽略工具错误构建和诊断

## 代码质量保证

### 1. Clippy 检查

所有代码修改都通过了 `cargo clippy --all-features --all-targets -- -D warnings` 检查，确保代码质量符合项目标准。

### 2. 测试通过

所有单元测试和集成测试都通过，确保功能的正确性和稳定性。

### 3. 文档覆盖

所有新增的公共方法和结构都添加了完整的文档注释，包括参数说明、返回值说明和使用示例。

## 后续优化建议

### 1. 性能优化

- 考虑添加诊断结果的缓存机制，避免重复诊断
- 优化诊断执行的性能，特别是深度诊断场景

### 2. 功能扩展

- 添加更多系统指标的监控和诊断
- 实现诊断结果的历史记录和趋势分析
- 支持自定义诊断规则和阈值

### 3. 用户体验

- 添加更丰富的诊断报告格式，如图表和可视化
- 实现诊断结果的导出功能，支持多种格式
- 添加诊断建议的自动化修复功能

## 总结

通过本次工作，我们成功为 AiExecUnitBuilder 添加了诊断配置支持，实现了以下目标：

1. 扩展了 AiExecUnitBuilder，支持诊断配置构建
2. 为 AiExecUnit 添加了多种诊断执行方法
3. 实现了完整的错误处理机制
4. 提供了用户友好的诊断报告格式化
5. 确保了代码质量和测试覆盖

这些改进为系统诊断工具的开发奠定了坚实基础，使得用户可以方便地使用 Orion AI 进行系统性能分析和问题诊断。