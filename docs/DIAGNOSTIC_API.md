# Orion AI 诊断功能 API 文档

## 概述

Orion AI 提供了强大的系统诊断功能，支持多级别的系统诊断和自定义诊断配置。本文档详细介绍了诊断功能的 API 使用方法。

## 核心类型

### DiagnosticConfig

```rust
pub struct DiagnosticConfig {
    pub check_basic_info: bool,      // 检查基本信息
    pub check_processes: bool,       // 检查进程信息
    pub check_io_performance: bool,  // 检查I/O性能
    pub check_network: bool,         // 检查网络状态
    pub timeout_seconds: u64,        // 超时时间
    pub sampling_interval: u64,      // 采样间隔
    pub sampling_count: u64,         // 采样次数
}
```

#### 字段说明

- `check_basic_info`: 是否检查系统基本信息（系统名称、版本、运行时间等）
- `check_processes`: 是否检查进程信息（进程列表、资源使用等）
- `check_io_performance`: 是否检查I/O性能（磁盘读写、网络I/O等）
- `check_network`: 是否检查网络状态（连接数、带宽使用等）
- `timeout_seconds`: 诊断操作的超时时间（秒）
- `sampling_interval`: 采样间隔（秒），用于性能监控
- `sampling_count`: 采样次数，用于性能监控

#### 默认值

```rust
impl Default for DiagnosticConfig {
    fn default() -> Self {
        Self {
            check_basic_info: true,
            check_processes: true,
            check_io_performance: true,
            check_network: true,
            timeout_seconds: 30,
            sampling_interval: 1,
            sampling_count: 3,
        }
    }
}
```

### DiagnosticDepth

```rust
pub enum DiagnosticDepth {
    Quick,      // 快速诊断
    Standard,   // 标准诊断
    Advanced,   // 深度诊断
}
```

#### 变体说明

- `Quick`: 快速诊断，检查系统基本状态，执行时间 < 1秒
- `Standard`: 标准诊断，全面检查系统性能和资源使用，执行时间 2-5秒
- `Advanced`: 深度诊断，深度分析系统性能瓶颈和问题根源，执行时间 5-10秒

### DiagnosticReport

```rust
pub struct DiagnosticReport {
    pub timestamp: DateTime<Utc>,
    pub system_info: SystemInfo,
    pub performance_metrics: PerformanceMetrics,
    pub issues: Vec<Issue>,
    pub recommendations: Vec<String>,
    pub execution_summary: ExecutionSummary,
}
```

#### 字段说明

- `timestamp`: 诊断报告生成时间
- `system_info`: 系统基本信息
- `performance_metrics`: 性能指标
- `issues`: 发现的问题列表
- `recommendations`: 建议列表
- `execution_summary`: 执行摘要

## AiExecUnitBuilder API

### 构造函数

```rust
pub fn new(dict: EnvDict) -> Self
```

创建一个新的 AiExecUnitBuilder 实例。

#### 参数

- `dict`: 环境变量字典

#### 示例

```rust
use orion_ai::*;
use orion_variate::vars::EnvDict;

let dict = EnvDict::new();
let builder = AiExecUnitBuilder::new(dict);
```

### 诊断配置方法

#### with_diagnostic_config

```rust
pub fn with_diagnostic_config(mut self, config: DiagnosticConfig) -> Self
```

设置诊断配置。

#### 参数

- `config`: 诊断配置

#### 示例

```rust
use orion_ai::*;
use orion_variate::vars::EnvDict;

let dict = EnvDict::new();
let config = DiagnosticConfig {
    check_basic_info: true,
    check_processes: true,
    check_io_performance: true,
    check_network: true,
    timeout_seconds: 20,
    sampling_interval: 2,
    sampling_count: 5,
};

let builder = AiExecUnitBuilder::new(dict)
    .with_diagnostic_config(config);
```

#### with_diagnostic_depth

```rust
pub fn with_diagnostic_depth(mut self, depth: DiagnosticDepth) -> Self
```

设置诊断深度。

#### 参数

- `depth`: 诊断深度

#### 示例

```rust
use orion_ai::*;
use orion_variate::vars::EnvDict;

let dict = EnvDict::new();
let builder = AiExecUnitBuilder::new(dict)
    .with_diagnostic_depth(DiagnosticDepth::Standard);
```

## AiExecUnit API

### 诊断执行方法

#### execute_diagnosis

```rust
pub async fn execute_diagnosis(&self, depth: DiagnosticDepth) -> AiResult<DiagnosticReport>
```

执行指定深度的诊断。

#### 参数

- `depth`: 诊断深度

#### 返回值

- `AiResult<DiagnosticReport>`: 诊断报告

#### 示例

```rust
use orion_ai::*;

let report = exec_unit.execute_diagnosis(DiagnosticDepth::Standard).await?;
println!("诊断报告: {}", report.formatted_report());
```

#### execute_diagnosis_with_config

```rust
pub async fn execute_diagnosis_with_config(&self, config: DiagnosticConfig) -> AiResult<DiagnosticReport>
```

使用自定义配置执行诊断。

#### 参数

- `config`: 诊断配置

#### 返回值

- `AiResult<DiagnosticReport>`: 诊断报告

#### 示例

```rust
use orion_ai::*;

let config = DiagnosticConfig {
    check_basic_info: true,
    check_processes: true,
    check_io_performance: true,
    check_network: true,
    timeout_seconds: 20,
    sampling_interval: 2,
    sampling_count: 5,
};

let report = exec_unit.execute_diagnosis_with_config(config).await?;
println!("诊断报告: {}", report.formatted_report());
```

#### quick_health_check

```rust
pub async fn quick_health_check(&self) -> AiResult<DiagnosticReport>
```

执行快速健康检查。

#### 返回值

- `AiResult<DiagnosticReport>`: 诊断报告

#### 示例

```rust
use orion_ai::*;

let report = exec_unit.quick_health_check().await?;
println!("快速健康检查报告: {}", report.formatted_report());
```

#### standard_diagnosis

```rust
pub async fn standard_diagnosis(&self) -> AiResult<DiagnosticReport>
```

执行标准诊断。

#### 返回值

- `AiResult<DiagnosticReport>`: 诊断报告

#### 示例

```rust
use orion_ai::*;

let report = exec_unit.standard_diagnosis().await?;
println!("标准诊断报告: {}", report.formatted_report());
```

#### deep_analysis

```rust
pub async fn deep_analysis(&self) -> AiResult<DiagnosticReport>
```

执行深度分析。

#### 返回值

- `AiResult<DiagnosticReport>`: 诊断报告

#### 示例

```rust
use orion_ai::*;

let report = exec_unit.deep_analysis().await?;
println!("深度分析报告: {}", report.formatted_report());
```

## DiagnosticReport API

### 格式化方法

#### formatted_report

```rust
pub fn formatted_report(&self) -> String
```

生成用户友好的诊断报告。

#### 返回值

- `String`: 格式化的诊断报告

#### 示例

```rust
use orion_ai::*;

let report = exec_unit.standard_diagnosis().await?;
println!("诊断报告:\n{}", report.formatted_report());
```

#### to_json

```rust
pub fn to_json(&self) -> String
```

生成JSON格式的诊断报告。

#### 返回值

- `String`: JSON格式的诊断报告

#### 示例

```rust
use orion_ai::*;

let report = exec_unit.standard_diagnosis().await?;
println!("JSON格式诊断报告:\n{}", report.to_json());
```

## 错误处理

### DiagnosisError

```rust
pub enum AiErrReason {
    // ... 其他错误类型
    DiagnosisError(String),  // 诊断错误
}
```

#### from_diagnosis

```rust
pub fn from_diagnosis(msg: String) -> Self
```

将诊断错误转换为系统错误。

#### 参数

- `msg`: 错误消息

#### 返回值

- `OrionAiReason`: 系统错误

#### 示例

```rust
use orion_ai::*;

let error = OrionAiReason::from_diagnosis("诊断超时".to_string());
```

## 使用示例

### 完整示例

```rust
use orion_ai::*;
use orion_variate::vars::EnvDict;

#[tokio::main]
async fn main() -> AiResult<()> {
    // 初始化全局函数注册表
    GlobalFunctionRegistry::initialize()?;

    // 创建环境变量字典
    let dict = EnvDict::new();

    // 创建带有诊断配置的执行单元
    let exec_unit = AiExecUnitBuilder::new(dict)
        .with_role("developer")
        .with_diagnostic_depth(DiagnosticDepth::Standard)
        .build()?;

    // 执行诊断
    let report = exec_unit.standard_diagnosis().await?;

    // 输出诊断报告
    println!("诊断报告:\n{}", report.formatted_report());

    Ok(())
}
```

### 自定义配置示例

```rust
use orion_ai::*;
use orion_variate::vars::EnvDict;

#[tokio::main]
async fn main() -> AiResult<()> {
    // 初始化全局函数注册表
    GlobalFunctionRegistry::initialize()?;

    // 创建环境变量字典
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
        .with_diagnostic_config(config.clone())
        .build()?;

    // 执行诊断
    let report = exec_unit.execute_diagnosis_with_config(config).await?;

    // 输出诊断报告
    println!("诊断报告:\n{}", report.formatted_report());

    Ok(())
}
```

### 多级别诊断示例

```rust
use orion_ai::*;
use orion_variate::vars::EnvDict;

#[tokio::main]
async fn main() -> AiResult<()> {
    // 初始化全局函数注册表
    GlobalFunctionRegistry::initialize()?;

    // 创建环境变量字典
    let dict = EnvDict::new();

    // 创建执行单元
    let exec_unit = AiExecUnitBuilder::new(dict)
        .with_role("developer")
        .with_diagnostic_depth(DiagnosticDepth::Advanced)
        .build()?;

    // 执行快速健康检查
    let quick_report = exec_unit.quick_health_check().await?;
    println!("快速健康检查报告:\n{}", quick_report.formatted_report());

    // 执行标准诊断
    let standard_report = exec_unit.standard_diagnosis().await?;
    println!("标准诊断报告:\n{}", standard_report.formatted_report());

    // 执行深度分析
    let deep_report = exec_unit.deep_analysis().await?;
    println!("深度分析报告:\n{}", deep_report.formatted_report());

    Ok(())
}
```

## 最佳实践

### 1. 选择合适的诊断深度

根据不同的使用场景选择合适的诊断深度：

- **日常监控**: 使用 `DiagnosticDepth::Quick` 进行快速检查
- **问题排查**: 使用 `DiagnosticDepth::Standard` 进行全面诊断
- **深度分析**: 使用 `DiagnosticDepth::Advanced` 进行深度分析

### 2. 自定义诊断配置

根据具体需求自定义诊断配置：

```rust
let config = DiagnosticConfig {
    check_basic_info: true,      // 总是检查基本信息
    check_processes: true,       // 检查进程信息
    check_io_performance: false,  // 不检查I/O性能（如果不需要）
    check_network: false,         // 不检查网络状态（如果不需要）
    timeout_seconds: 15,         // 设置较短的超时时间
    sampling_interval: 2,        // 设置采样间隔
    sampling_count: 3,           // 设置采样次数
};
```

### 3. 错误处理

正确处理诊断过程中可能出现的错误：

```rust
match exec_unit.standard_diagnosis().await {
    Ok(report) => {
        println!("诊断报告:\n{}", report.formatted_report());
    }
    Err(AiError::Reason(reason)) => {
        match reason.reason() {
            AiErrReason::DiagnosisError(msg) => {
                eprintln!("诊断错误: {}", msg);
            }
            _ => {
                eprintln!("其他错误: {}", reason);
            }
        }
    }
    Err(e) => {
        eprintln!("未知错误: {}", e);
    }
}
```

### 4. 性能考虑

- 避免频繁执行深度诊断，以免影响系统性能
- 在生产环境中，建议使用较长的超时时间
- 考虑使用缓存机制，避免重复诊断

## 总结

Orion AI 的诊断功能提供了灵活的系统诊断能力，支持多级别的诊断和自定义配置。通过合理使用这些 API，可以有效地监控系统状态、排查问题和优化性能。