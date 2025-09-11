# Orion AI 项目文档

## 🚀 项目概览

Orion AI 是一个 Rust 实现的智能 AI 执行引擎，支持多 AI 提供商、函数调用、线程记录和角色配置管理。提供统一的接口与不同的 AI 模型交互。

## 📁 代码架构

### 1. 核心模块结构

```
src/
├── client/           # 客户端实现（OpenAI/DeepSeek等）
├── config/           # 配置系统（角色、模型、规则）
├── exec_unit/        # 执行单元（AiExecUnit）
├── func/             # 函数调用系统
│   ├── global/       # 全局注册管理
│   ├── system/       # 系统工具函数
│   ├── git.rs        # Git操作函数
│   ├── registry.rs   # 函数注册器
│   └── executor.rs   # 函数执行器
├── providers/        # AI提供商实现
├── thread/           # 线程记录系统
├── types/            # 类型定义
└── router/           # 请求路由系统
```

### 2. 关键组件

#### 客户端层 (`src/client/`)
- **AiClient**: 统一的AI请求客户端
- **AiCoreClient**: 核心客户端实现
- **MockClient**: 测试用的模拟客户端

#### 配置系统 (`src/config/`)
- **RoleConfigManager**: 角色配置管理
- **ConfigLoader**: 配置加载器
- **ProviderConfig**: 提供商配置
- **RoutingRules**: 路由规则

#### 函数调用系统 (`src/func/`)
- **FunctionRegistry**: 函数注册表
- **FunctionExecutor**: 函数执行器trait
- **GlobalFunctionRegistry**: 全局注册管理中心

#### 执行单元 (`src/exec_unit/`)
- **AiExecUnit**: 核心执行单元，封装：
  - AI客户端
  - 工具函数
  - 角色配置
  - 线程记录
  - 诊断配置（可选）
- **AiExecUnitBuilder**: 构建器模式，支持：
  - 基础配置构建
  - 诊断配置构建
  - 链式配置方法

#### 线程记录 (`src/thread/`)
- **ThreadClient**: 线程客户端包装
- **ThreadFileManager**: 文件管理器
- **ThreadConfig**: 线程配置

### 3. 系统工具

#### 支持的函数工具

**Git操作（`git/`）:**
- `git-status` - 查看仓库状态
- `git-add <file>` - 添加文件到暂存区
- `git-commit <msg>` - 提交更改
- `git-push` - 推送提交
- `git-diff <options>` - 查看差异
- `git-log <count>` - 查看提交历史

**文件系统（`system/`）:**
- `fs-ls <path>` - 列出目录内容
- `fs-cat <file>` - 查看文件内容
- `fs-find <pattern>` - 搜索文件
- `fs-pwd` - 当前工作目录

**系统信息（`system/`）:**
- `sys-uname` - 系统信息
- `sys-ps` - 进程列表
- `sys-df` - 磁盘使用情况
- `net-ping <host>` - 网络连通性测试

**系统诊断（`system/`）:**
- `sys-uptime` - 系统运行时间和负载平均值
- `sys-meminfo` - 内存使用详细信息
- `sys-top` - 系统资源概览（CPU、内存、进程）
- `sys-cpuload` - CPU使用率详细统计
- `sys-proc-top` - 高资源消耗进程列表（按CPU/内存排序）
- `sys-proc-stats` - 进程统计信息（总数、状态分布）
- `sys-iostat` - I/O统计信息（读写速度、等待时间）
- `sys-netstat` - 网络连接统计（连接数、带宽使用）
- `sys-diagnose` - 综合系统诊断（多维度分析）

## 🔧 快速开始

### 1. 安装依赖
```bash
cargo build --release
```

### 2. 基本使用示例

#### 基础使用
```rust
use orion_ai::*;

#[tokio::main]
async fn main() {
    // 初始化全局函数注册表
    GlobalFunctionRegistry::initialize().unwrap();
    
    // 创建AI执行单元
    let exec_unit = AiExecUnitBuilder::new(load_config())
        .with_role("developer")
        .with_tools(vec!["git-status".to_string(), "fs-ls".to_string()])
        .build();
    
    // 执行智能请求
    let result = exec_unit.smart_request(
        "查看当前git状态".to_string()
    ).await.unwrap();
    
    println!("结果: {}", result);
}
```

#### 诊断功能使用
```rust
use orion_ai::*;

#[tokio::main]
async fn main() {
    // 初始化全局函数注册表
    GlobalFunctionRegistry::initialize().unwrap();
    
    // 创建带有诊断配置的AI执行单元
    let exec_unit = AiExecUnitBuilder::new(load_config())
        .with_role("developer")
        .with_diagnostic_depth(DiagnosticDepth::Standard)
        .build();
    
    // 执行系统诊断
    let report = exec_unit.standard_diagnosis().await.unwrap();
    println!("诊断报告:\n{}", report.formatted_report());
    
    // 或者使用自定义诊断配置
    let config = DiagnosticConfig {
        check_basic_info: true,
        check_processes: true,
        check_io_performance: true,
        check_network: true,
        timeout_seconds: 20,
        sampling_interval: 2,
        sampling_count: 5,
    };
    
    let exec_unit = AiExecUnitBuilder::new(load_config())
        .with_role("developer")
        .with_diagnostic_config(config)
        .build();
    
    let report = exec_unit.execute_diagnosis_with_config(config).await.unwrap();
    println!("自定义诊断报告:\n{}", report.formatted_report());
}
```

### 3. 配置结构

配置文件位于 `./_gal/` 目录：

```
_gal/
├── ai.yml           # 主配置文件
├── ai-roles.yml    # 角色配置
├── ai-roles/[role]/ # 角色规则文件
├── env/            # 环境变量
└── secret/         # 密钥文件
```

## 📊 性能特点

- **线程安全**: 使用 `OnceLock<Arc<RwLock>>` 实现
- **零拷贝**: 基于 Rust 的所有权系统
- **异步执行**: 基于 Tokio runtime
- **可选缓存**: 响应缓存支持

## 🔐 安全配置

### 密钥管理
环境变量或 `_gal/secret/` 文件：
- `OPENAI_API_KEY`
- `DEEPSEEK_API_KEY`
- `ZHIPUAI_API_KEY`
- `MOONSHOT_API_KEY`

## 🔍 诊断功能架构设计

### 1. 分级诊断策略

Orion AI 实现了三级诊断策略，根据不同场景提供不同深度的系统诊断：

#### 快速诊断 (Quick)
- **目标**: 快速检查系统基本状态
- **检查项**: 系统基本信息、关键进程状态、基本资源使用
- **执行时间**: < 1秒
- **适用场景**: 日常快速检查、系统响应缓慢初步排查

#### 标准诊断 (Standard)
- **目标**: 全面检查系统性能和资源使用
- **检查项**: 系统详细信息、进程分析、I/O性能、网络状态
- **执行时间**: 2-5秒
- **适用场景**: 系统卡顿问题排查、性能问题分析

#### 深度诊断 (Advanced)
- **目标**: 深度分析系统性能瓶颈和问题根源
- **检查项**: 全面的系统监控、详细的进程分析、历史数据对比、趋势分析
- **执行时间**: 5-10秒
- **适用场景**: 复杂问题排查、系统优化、性能调优

### 2. 诊断配置系统

#### DiagnosticConfig 结构
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

#### 诊断深度枚举
```rust
pub enum DiagnosticDepth {
    Quick,      // 快速诊断
    Standard,   // 标准诊断
    Advanced,   // 深度诊断
}
```

### 3. 诊断执行流程

#### 诊断执行器
- **DiagnosisExecutor**: 基础诊断功能执行器
- **MonitorExecutor**: 进程监控功能执行器
- **AnalysisExecutor**: 性能分析功能执行器

#### 诊断执行方法
- `execute_diagnosis(depth: DiagnosticDepth)`: 执行指定深度的诊断
- `execute_diagnosis_with_config(config: DiagnosticConfig)`: 使用自定义配置执行诊断
- `quick_health_check()`: 执行快速健康检查
- `standard_diagnosis()`: 执行标准诊断
- `deep_analysis()`: 执行深度分析

### 4. 诊断结果处理

#### 诊断报告结构
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

#### 报告格式化
- `formatted_report()`: 生成用户友好的诊断报告
- `to_json()`: 生成JSON格式的诊断报告
- `to_chinese()`: 生成中文格式的诊断报告

### 5. 错误处理机制

#### 诊断错误类型
```rust
pub enum AiErrReason {
    // ... 其他错误类型
    DiagnosisError(String),  // 诊断错误
}
```

#### 错误转换
- `OrionAiReason::from_diagnosis(msg: String)`: 将诊断错误转换为系统错误

### 6. 诊断功能集成

#### AiExecUnit 集成
- 在 AiExecUnit 结构体中添加 `diagnostic_config` 字段
- 提供诊断执行方法
- 支持诊断结果的格式化输出

#### AiExecUnitBuilder 集成
- 在 AiExecUnitBuilder 结构体中添加 `diagnostic_config` 字段
- 提供 `with_diagnostic_config()` 和 `with_diagnostic_depth()` 方法
- 更新 `build()` 和 `build_ignoring_tool_errors()` 方法，支持诊断配置

### 7. 诊断功能优化

#### 性能优化
- 诊断结果缓存机制
- 并行执行诊断任务
- 智能采样策略

#### 用户体验优化
- 多语言支持（中文/英文）
- 可视化报告格式
- 交互式诊断界面