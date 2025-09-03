# Orion AI 个人AI系统说明文档

## 简介
Orion AI 是一个功能强大的Rust AI助手系统，支持多AI提供商环境、函数调用、Git操作和系统工具。

## 核心特性
- **多AI提供商支持**: DeepSeek, OpenAI, 智谱GLM, 月之暗面Kimi
- **函数调用**: Git操作、文件系统、系统信息、网络测试
- **线程记录**: 完整会话历史记录
- **角色配置**: 专家级角色规则系统
- **安全机制**: 环境变量分离、路径验证、命令过滤

## 配置结构
- 主配置: _gal/ai.yml
- 角色配置: _gal/ai-roles.yml  
- 规则文件: _gal/ai-rules/{role}/
- 密钥配置: ~/.galaxy/sec_value.yml

## 支持的函数工具
- Git: status, add, commit, push, pull, diff, log
- 系统: ls, cat, find, pwd, cd
- 网络: ping主机、测试连通性

## 使用方式
实时AI助手: cargo run -p orion-ai
集成开发: 使用AiExecUnit构建器模式
> build docs/README.md
