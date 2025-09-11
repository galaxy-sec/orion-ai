//! 系统诊断示例程序
//! 
//! 这个示例程序展示了如何使用 Orion AI 的诊断功能来检查系统状态。

use orion_ai::*;
use orion_variate::vars::EnvDict;
use orion_ai::types::{DiagnosticDepth, DiagnosticConfig};

#[tokio::main]
async fn main() -> AiResult<()> {
    // 初始化全局函数注册表
    GlobalFunctionRegistry::initialize()?;

    // 创建环境变量字典
    let dict = EnvDict::new();

    println!("=== Orion AI 系统诊断示例 ===\n");

    // 示例1: 使用诊断深度配置
    println!("1. 使用诊断深度配置:");
    example_with_diagnostic_depth(&dict).await?;

    // 示例2: 使用自定义诊断配置
    println!("\n2. 使用自定义诊断配置:");
    example_with_custom_config(&dict).await?;

    // 示例3: 执行不同级别的诊断
    println!("\n3. 执行不同级别的诊断:");
    example_different_diagnosis_levels(&dict).await?;

    Ok(())
}

/// 示例1: 使用诊断深度配置
async fn example_with_diagnostic_depth(dict: &EnvDict) -> AiResult<()> {
    // 创建带有标准诊断深度的执行单元
    let exec_unit = AiExecUnitBuilder::new(dict.clone())
        .with_role("developer")
        .with_diagnostic_depth(DiagnosticDepth::Standard)
        .build()?;

    // 执行标准诊断
    let report = exec_unit.standard_diagnosis().await?;
    println!("标准诊断结果:\n{}", report.formatted_report());

    Ok(())
}

/// 示例2: 使用自定义诊断配置
async fn example_with_custom_config(dict: &EnvDict) -> AiResult<()> {
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
    let exec_unit = AiExecUnitBuilder::new(dict.clone())
        .with_role("developer")
        .with_diagnostic_config(config.clone())
        .build()?;

    // 执行自定义诊断
    let report = exec_unit.execute_diagnosis_with_config(config).await?;
    println!("自定义诊断结果:\n{}", report.formatted_report());

    Ok(())
}

/// 示例3: 执行不同级别的诊断
async fn example_different_diagnosis_levels(dict: &EnvDict) -> AiResult<()> {
    // 创建带有高级诊断深度的执行单元
    let exec_unit = AiExecUnitBuilder::new(dict.clone())
        .with_role("developer")
        .with_diagnostic_depth(DiagnosticDepth::Advanced)
        .build()?;

    // 快速健康检查
    let quick_report = exec_unit.quick_health_check().await?;
    println!("快速健康检查结果:\n{}", quick_report.formatted_report());

    // 标准诊断
    let standard_report = exec_unit.standard_diagnosis().await?;
    println!("标准诊断结果:\n{}", standard_report.formatted_report());

    // 深度分析
    let deep_report = exec_unit.deep_analysis().await?;
    println!("深度分析结果:\n{}", deep_report.formatted_report());

    Ok(())
}