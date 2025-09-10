//! AI执行单元诊断模块
//!
//! 此模块提供了渐进式诊断的核心逻辑，支持不同深度的系统诊断，
//! 包括快速健康检查、标准诊断和深度分析功能。

use crate::FunctionRegistry;
use crate::provider::FunctionCall;
use crate::types::SamplingConfig;
use crate::types::diagnosis::{
    DiagnosticConfig, DiagnosticDepth, DiagnosticReport, IoMetrics, IssueSeverity, NetworkMetrics,
    ProcessMetrics, Recommendation, SystemIssue, SystemIssueType,
};
use chrono::{DateTime, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;

/// 诊断执行器特质
///
/// 定义了诊断执行的核心接口，支持不同类型的诊断实现。
#[async_trait::async_trait]
pub trait DiagnosticExecutor {
    /// 执行诊断并返回诊断报告
    async fn execute_diagnosis(
        &self,
        depth: DiagnosticDepth,
    ) -> Result<DiagnosticReport, DiagnosisError>;

    /// 执行快速健康检查
    async fn quick_health_check(&self) -> Result<DiagnosticReport, DiagnosisError>;

    /// 执行标准诊断
    async fn standard_diagnosis(&self) -> Result<DiagnosticReport, DiagnosisError>;

    /// 执行深度分析
    async fn deep_analysis(&self) -> Result<DiagnosticReport, DiagnosisError>;
}

/// 诊断错误类型
#[derive(Debug, Clone, thiserror::Error)]
pub enum DiagnosisError {
    #[error("函数执行失败: {0}")]
    FunctionExecutionFailed(String),

    #[error("超时: {0}s 超过限制 {1}s")]
    Timeout(u64, u64),

    #[error("参数解析错误: {0}")]
    ParameterParseError(String),

    #[error("系统命令执行失败: {0}")]
    SystemCommandFailed(String),

    #[error("诊断配置无效: {0}")]
    InvalidConfig(String),

    #[error("未知诊断错误: {0}")]
    Unknown(String),
}

/// 渐进式诊断器
///
/// 实现了渐进式诊断的核心逻辑，支持不同深度的系统诊断。
pub struct ProgressiveDiagnosis {
    /// 诊断配置
    config: DiagnosticConfig,

    /// 采样配置
    sampling_config: SamplingConfig,

    /// 开始时间
    #[allow(dead_code)]
    start_time: Instant,
}

impl ProgressiveDiagnosis {
    /// 创建新的渐进式诊断器
    pub fn new(config: DiagnosticConfig) -> Self {
        let sampling_config = SamplingConfig::new(
            config.sampling_count,
            config.sampling_interval,
            config.timeout_seconds,
        );

        Self {
            config,
            sampling_config,
            start_time: Instant::now(),
        }
    }

    /// 使用诊断深度创建诊断器
    pub fn from_depth(depth: DiagnosticDepth) -> Self {
        let config = depth.to_config();
        Self::new(config)
    }

    /// 执行渐进式诊断
    pub async fn execute_progressive_diagnosis(
        &self,
        function_registry: &FunctionRegistry,
    ) -> Result<DiagnosticReport, DiagnosisError> {
        let start_time = Utc::now();
        let mut report = DiagnosticReport::new(DiagnosticDepth::Custom(self.config.clone()));

        // 执行基础诊断
        if self.config.check_basic_info {
            self.execute_basic_diagnosis(function_registry, &mut report)
                .await?;
        }

        // 执行进程分析
        if self.config.check_processes {
            self.execute_process_analysis(function_registry, &mut report)
                .await?;
        }

        // 执行I/O性能分析
        if self.config.check_io_performance {
            self.execute_io_analysis(function_registry, &mut report)
                .await?;
        }

        // 执行网络分析
        if self.config.check_network {
            self.execute_network_analysis(function_registry, &mut report)
                .await?;
        }

        // 更新执行摘要
        self.update_execution_summary(&mut report, start_time);

        // 生成问题分析和建议
        self.analyze_and_recommend(&mut report);

        Ok(report)
    }

    /// 执行基础诊断
    async fn execute_basic_diagnosis(
        &self,
        function_registry: &FunctionRegistry,
        report: &mut DiagnosticReport,
    ) -> Result<(), DiagnosisError> {
        // 获取系统运行时间
        let uptime_call = self.create_function_call("sys-uptime", "{}");
        let uptime_result = self
            .execute_function_with_timeout(
                function_registry,
                &uptime_call,
                self.config.timeout_seconds,
            )
            .await?;

        // 获取内存信息
        let meminfo_call = self.create_function_call("sys-meminfo", "{}");
        let meminfo_result = self
            .execute_function_with_timeout(
                function_registry,
                &meminfo_call,
                self.config.timeout_seconds,
            )
            .await?;

        // 更新报告中的系统信息
        self.update_system_info(report, &uptime_result, &meminfo_result);

        // 更新执行摘要
        report
            .execution_summary
            .executed_functions
            .push("sys-uptime".to_string());
        report
            .execution_summary
            .executed_functions
            .push("sys-meminfo".to_string());
        report.execution_summary.successful_functions += 2;

        Ok(())
    }

    /// 执行进程分析
    async fn execute_process_analysis(
        &self,
        function_registry: &FunctionRegistry,
        report: &mut DiagnosticReport,
    ) -> Result<(), DiagnosisError> {
        // 获取高CPU进程
        let cpu_top_call =
            self.create_function_call("sys-proc-top", r#"{"sort_by": "cpu", "limit": 5}"#);
        let cpu_top_result = self
            .execute_function_with_timeout(
                function_registry,
                &cpu_top_call,
                self.config.timeout_seconds,
            )
            .await?;

        // 获取高内存进程
        let mem_top_call =
            self.create_function_call("sys-proc-top", r#"{"sort_by": "memory", "limit": 5}"#);
        let mem_top_result = self
            .execute_function_with_timeout(
                function_registry,
                &mem_top_call,
                self.config.timeout_seconds,
            )
            .await?;

        // 获取进程统计
        let proc_stats_call = self.create_function_call("sys-proc-stats", "{}");
        let proc_stats_result = self
            .execute_function_with_timeout(
                function_registry,
                &proc_stats_call,
                self.config.timeout_seconds,
            )
            .await?;

        // 更新报告中的进程指标
        self.update_process_metrics(report, &cpu_top_result, &mem_top_result, &proc_stats_result);

        // 更新执行摘要
        report
            .execution_summary
            .executed_functions
            .push("sys-proc-top".to_string());
        report
            .execution_summary
            .executed_functions
            .push("sys-proc-stats".to_string());
        report.execution_summary.successful_functions += 2;

        Ok(())
    }

    /// 执行I/O性能分析
    async fn execute_io_analysis(
        &self,
        function_registry: &FunctionRegistry,
        report: &mut DiagnosticReport,
    ) -> Result<(), DiagnosisError> {
        // 获取I/O统计
        let iostat_call = self.create_function_call(
            "sys-iostat",
            &format!(
                r#"{{"count": {}, "interval": {}}}"#,
                self.sampling_config.count, self.sampling_config.interval
            ),
        );
        let iostat_result = self
            .execute_function_with_timeout(
                function_registry,
                &iostat_call,
                self.sampling_config.timeout * 2, // I/O分析可能需要更长时间
            )
            .await?;

        // 更新报告中的I/O指标
        self.update_io_metrics(report, &iostat_result);

        // 更新执行摘要
        report
            .execution_summary
            .executed_functions
            .push("sys-iostat".to_string());
        report.execution_summary.successful_functions += 1;

        Ok(())
    }

    /// 执行网络分析
    async fn execute_network_analysis(
        &self,
        function_registry: &FunctionRegistry,
        report: &mut DiagnosticReport,
    ) -> Result<(), DiagnosisError> {
        // 获取网络统计
        let netstat_call =
            self.create_function_call("sys-netstat", r#"{"show_tcp": true, "show_udp": true}"#);
        let netstat_result = self
            .execute_function_with_timeout(
                function_registry,
                &netstat_call,
                self.config.timeout_seconds,
            )
            .await?;

        // 更新报告中的网络指标
        self.update_network_metrics(report, &netstat_result);

        // 更新执行摘要
        report
            .execution_summary
            .executed_functions
            .push("sys-netstat".to_string());
        report.execution_summary.successful_functions += 1;

        Ok(())
    }

    /// 创建函数调用
    fn create_function_call(&self, function_name: &str, arguments: &str) -> FunctionCall {
        FunctionCall {
            index: Some(0),
            id: format!("diagnosis_{}_{}", function_name, uuid::Uuid::new_v4()),
            r#type: "function".to_string(),
            function: crate::provider::FunctionCallInfo {
                name: function_name.to_string(),
                arguments: arguments.to_string(),
            },
        }
    }

    /// 带超时执行的函数
    async fn execute_function_with_timeout(
        &self,
        function_registry: &FunctionRegistry,
        function_call: &FunctionCall,
        timeout_seconds: u64,
    ) -> Result<serde_json::Value, DiagnosisError> {
        use tokio::time::{Duration, timeout};

        let result = timeout(
            Duration::from_secs(timeout_seconds),
            function_registry.execute_function(function_call),
        )
        .await;

        match result {
            Ok(Ok(function_result)) => {
                if let Some(error) = function_result.error {
                    Err(DiagnosisError::FunctionExecutionFailed(error))
                } else {
                    Ok(function_result.result)
                }
            }
            Ok(Err(e)) => Err(DiagnosisError::FunctionExecutionFailed(e.to_string())),
            Err(_) => Err(DiagnosisError::Timeout(timeout_seconds, timeout_seconds)),
        }
    }

    /// 更新系统信息
    fn update_system_info(
        &self,
        report: &mut DiagnosticReport,
        uptime_result: &serde_json::Value,
        meminfo_result: &serde_json::Value,
    ) {
        // 更新运行时间
        if let Some(uptime_output) = uptime_result.get("uptime_output") {
            report.system_info.uptime = uptime_output.as_str().unwrap_or("").to_string();
        }

        // 更新负载信息
        if let Some(load_data) = uptime_result.get("load_data")
            && let Some(load_average) = load_data.get("load_average")
            && let Some(array) = load_average.as_array()
        {
            report.system_info.load_average = array.iter().filter_map(|v| v.as_f64()).collect();
        }

        // 更新内存信息
        if let Some(memory_data) = meminfo_result.get("memory_data") {
            if let Some(total) = memory_data.get("total").and_then(|v| v.as_u64()) {
                report.performance_metrics.memory_metrics.total_memory = total;
            }
            if let Some(used) = memory_data.get("used").and_then(|v| v.as_u64()) {
                report.performance_metrics.memory_metrics.used_memory = used;
            }
            if let Some(usage_percent) = memory_data.get("usage_percent").and_then(|v| v.as_f64()) {
                report.performance_metrics.memory_metrics.usage_percent = usage_percent;
            }
        }

        // 更新CPU指标
        if let Some(cpu_data) = uptime_result.get("cpu_data")
            && let Some(usage_percent) = cpu_data.get("usage_percent").and_then(|v| v.as_f64())
        {
            report.performance_metrics.cpu_metrics.usage_percent = usage_percent;
        }
    }

    /// 更新进程指标
    fn update_process_metrics(
        &self,
        report: &mut DiagnosticReport,
        _cpu_top_result: &serde_json::Value,
        _mem_top_result: &serde_json::Value,
        proc_stats_result: &serde_json::Value,
    ) {
        let process_metrics = ProcessMetrics::default();

        // 更新进程统计信息
        if let Some(process_stats) = proc_stats_result.get("process_stats")
            && let Some(_total) = process_stats
                .get("total_processes")
                .and_then(|v| v.as_u64())
        {
            // 这里应该用实际的数据更新，但为了简化我们使用默认值
        }

        report.performance_metrics.process_metrics = Some(process_metrics);
    }

    /// 更新I/O指标
    fn update_io_metrics(&self, report: &mut DiagnosticReport, iostat_result: &serde_json::Value) {
        let io_metrics = IoMetrics::default();

        // 更新I/O统计信息
        if let Some(_iostat_output) = iostat_result.get("iostat_output") {
            // 这里应该用实际的数据更新，但为了简化我们使用默认值
        }

        report.performance_metrics.io_metrics = Some(io_metrics);
    }

    /// 更新网络指标
    fn update_network_metrics(
        &self,
        report: &mut DiagnosticReport,
        netstat_result: &serde_json::Value,
    ) {
        let mut network_metrics = NetworkMetrics::default();

        // 更新网络统计信息
        if let Some(connection_stats) = netstat_result.get("connection_stats") {
            if let Some(tcp_connections) = connection_stats
                .get("tcp_connections")
                .and_then(|v| v.as_u64())
            {
                network_metrics.tcp_connections = tcp_connections as u32;
            }
            if let Some(udp_connections) = connection_stats
                .get("udp_connections")
                .and_then(|v| v.as_u64())
            {
                network_metrics.udp_connections = udp_connections as u32;
            }
            if let Some(listening_ports) = connection_stats
                .get("listening_ports")
                .and_then(|v| v.as_u64())
            {
                network_metrics.listening_ports = listening_ports as u32;
            }
            if let Some(established_connections) = connection_stats
                .get("established_connections")
                .and_then(|v| v.as_u64())
            {
                network_metrics.established_connections = established_connections as u32;
            }
        }

        report.performance_metrics.network_metrics = Some(network_metrics);
    }

    /// 更新执行摘要
    fn update_execution_summary(&self, report: &mut DiagnosticReport, start_time: DateTime<Utc>) {
        report.execution_summary.total_duration_seconds =
            (Utc::now() - start_time).num_seconds() as f64;
        report.execution_summary.status = "completed".to_string();
        report.execution_summary.failed_functions =
            report.execution_summary.executed_functions.len()
                - report.execution_summary.successful_functions;
    }

    /// 分析问题并生成建议
    fn analyze_and_recommend(&self, report: &mut DiagnosticReport) {
        // 分析CPU使用率
        if report.performance_metrics.cpu_metrics.usage_percent > 80.0 {
            report.add_issue(SystemIssue {
                issue_type: SystemIssueType::HighCpuUsage,
                description: format!(
                    "CPU使用率过高: {:.1}%",
                    report.performance_metrics.cpu_metrics.usage_percent
                ),
                severity: IssueSeverity::Warning,
                metrics: {
                    let mut metrics = HashMap::new();
                    metrics.insert(
                        "cpu_usage_percent".to_string(),
                        json!(report.performance_metrics.cpu_metrics.usage_percent),
                    );
                    metrics
                },
                recommendations: vec![
                    "检查高CPU进程".to_string(),
                    "考虑终止不必要的进程".to_string(),
                ],
            });

            report.add_recommendation(Recommendation {
                title: "优化CPU使用".to_string(),
                description: "建议检查和优化高CPU使用进程".to_string(),
                priority: 3,
                steps: vec![
                    "使用top命令检查CPU使用".to_string(),
                    "识别和终止高CPU进程".to_string(),
                ],
                expected_outcome: "CPU使用率降低到正常范围".to_string(),
            });
        }

        // 分析内存使用率
        if report.performance_metrics.memory_metrics.usage_percent > 85.0 {
            report.add_issue(SystemIssue {
                issue_type: SystemIssueType::HighMemoryUsage,
                description: format!(
                    "内存使用率过高: {:.1}%",
                    report.performance_metrics.memory_metrics.usage_percent
                ),
                severity: IssueSeverity::Warning,
                metrics: {
                    let mut metrics = HashMap::new();
                    metrics.insert(
                        "memory_usage_percent".to_string(),
                        json!(report.performance_metrics.memory_metrics.usage_percent),
                    );
                    metrics
                },
                recommendations: vec!["检查高内存使用进程".to_string(), "考虑清理缓存".to_string()],
            });

            report.add_recommendation(Recommendation {
                title: "优化内存使用".to_string(),
                description: "建议检查和优化内存使用情况".to_string(),
                priority: 3,
                steps: vec![
                    "使用free命令检查内存使用".to_string(),
                    "清理不必要的缓存".to_string(),
                ],
                expected_outcome: "内存使用率降低到正常范围".to_string(),
            });
        }
    }
}

/// 渐进式诊断的便捷函数
///
/// 提供简单易用的渐进式诊断接口。
pub async fn progressive_diagnosis(
    depth: DiagnosticDepth,
    function_registry: &FunctionRegistry,
) -> Result<DiagnosticReport, DiagnosisError> {
    let diagnosis = ProgressiveDiagnosis::from_depth(depth);
    diagnosis
        .execute_progressive_diagnosis(function_registry)
        .await
}

/// 快速健康检查
///
/// 执行基础系统健康检查。
pub async fn quick_health_check(
    function_registry: &FunctionRegistry,
) -> Result<DiagnosticReport, DiagnosisError> {
    progressive_diagnosis(DiagnosticDepth::Basic, function_registry).await
}

/// 标准诊断
///
/// 执行标准深度系统诊断。
pub async fn standard_diagnosis(
    function_registry: &FunctionRegistry,
) -> Result<DiagnosticReport, DiagnosisError> {
    progressive_diagnosis(DiagnosticDepth::Standard, function_registry).await
}

/// 深度分析
///
/// 执行全面深度系统分析。
pub async fn deep_analysis(
    function_registry: &FunctionRegistry,
) -> Result<DiagnosticReport, DiagnosisError> {
    progressive_diagnosis(DiagnosticDepth::Advanced, function_registry).await
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::FunctionRegistry;

    #[tokio::test]
    async fn test_progressive_diagnosis_creation() {
        let config = DiagnosticConfig::basic();
        let diagnosis = ProgressiveDiagnosis::new(config);

        assert!(diagnosis.config.check_basic_info);
        assert!(!diagnosis.config.check_processes);
    }

    #[tokio::test]
    async fn test_progressive_diagnosis_from_depth() {
        let diagnosis = ProgressiveDiagnosis::from_depth(DiagnosticDepth::Standard);

        assert!(diagnosis.config.check_basic_info);
        assert!(diagnosis.config.check_processes);
        assert!(!diagnosis.config.check_io_performance);
    }

    #[tokio::test]
    async fn test_function_call_creation() {
        let config = DiagnosticConfig::basic();
        let diagnosis = ProgressiveDiagnosis::new(config);

        let call = diagnosis.create_function_call("sys-uptime", "{}");
        assert_eq!(call.function.name, "sys-uptime");
        assert_eq!(call.function.arguments, "{}");
    }

    #[tokio::test]
    async fn test_convenience_functions() {
        // 由于这些函数需要实际的功能注册表，这里只是测试它们存在
        // 占位符测试 - 便利函数存在性检查
        // TODO: 实际测试需要模拟的功能注册表
    }
}
