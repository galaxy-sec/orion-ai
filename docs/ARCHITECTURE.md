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
- **AiExecUnitBuilder**: 构建器模式

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

## 🔧 快速开始

### 1. 安装依赖
```bash
cargo build --release
```

### 2. 基本使用示例

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
- `DEEPSEEK_API