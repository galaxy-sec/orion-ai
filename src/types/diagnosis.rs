//! 系统诊断相关类型定义
//!
//! 这个模块包含了系统诊断工具使用的核心类型定义，
//! 包括诊断模式、采样配置和诊断报告等。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 诊断模式枚举
///
/// 定义了系统诊断的不同深度模式，用于控制诊断的详细程度和执行时间。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy, Default)]
pub enum DiagnosticMode {
    /// 快速扫描模式 (200ms内完成)
    ///
    /// 执行基础系统健康检查：
    /// - 系统运行时间和负载
    /// - 内存使用情况概览
    /// - 基础进程信息
    Quick,

    /// 标准诊断模式 (1-2秒完成)
    ///
    /// 执行中等深度诊断：
    /// - 所有快速模式检查
    /// - 高资源消耗进程分析
    /// - 详细的系统资源统计
    #[default]
    Standard,

    /// 深度分析模式 (5-10秒完成)
    ///
    /// 执行全面系统诊断：
    /// - 所有标准模式检查
    /// - I/O性能统计
    /// - 网络连接分析
    /// - 综合性能评估
    Deep,
}

impl std::fmt::Display for DiagnosticMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticMode::Quick => write!(f, "quick"),
            DiagnosticMode::Standard => write!(f, "standard"),
            DiagnosticMode::Deep => write!(f, "deep"),
        }
    }
}

impl DiagnosticMode {
    /// 获取诊断模式的描述信息
    pub fn description(&self) -> &'static str {
        match self {
            DiagnosticMode::Quick => "快速系统扫描 (200ms内)",
            DiagnosticMode::Standard => "标准系统诊断 (1-2秒)",
            DiagnosticMode::Deep => "深度系统分析 (5-10秒)",
        }
    }

    /// 获取诊断模式的超时时间（秒）
    pub fn timeout_seconds(&self) -> u64 {
        match self {
            DiagnosticMode::Quick => 5,     // 5秒超时
            DiagnosticMode::Standard => 15, // 15秒超时
            DiagnosticMode::Deep => 30,     // 30秒超时
        }
    }

    /// 检查是否应该执行进程分析
    pub fn should_analyze_processes(&self) -> bool {
        matches!(self, DiagnosticMode::Standard | DiagnosticMode::Deep)
    }

    /// 检查是否应该执行I/O分析
    pub fn should_analyze_io(&self) -> bool {
        matches!(self, DiagnosticMode::Deep)
    }

    /// 检查是否应该执行网络分析
    pub fn should_analyze_network(&self) -> bool {
        matches!(self, DiagnosticMode::Deep)
    }
}

/// 采样配置结构体
///
/// 用于控制诊断过程中的采样行为，包括采样次数、间隔和超时设置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    /// 采样次数
    pub count: u32,

    /// 采样间隔（秒）
    pub interval: u32,

    /// 超时时间（秒）
    pub timeout: u64,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            count: 2,
            interval: 1,
            timeout: 10,
        }
    }
}

impl SamplingConfig {
    /// 创建新的采样配置
    pub fn new(count: u32, interval: u32, timeout: u64) -> Self {
        Self {
            count: count.clamp(1, 10),      // 限制采样次数
            interval: interval.clamp(1, 5), // 限制采样间隔
            timeout: timeout.clamp(5, 60),  // 限制超时时间
        }
    }

    /// 根据诊断模式生成自适应采样配置
    pub fn for_diagnostic_mode(mode: &DiagnosticMode) -> Self {
        match mode {
            DiagnosticMode::Quick => Self {
                count: 1,
                interval: 1,
                timeout: 5,
            },
            DiagnosticMode::Standard => Self {
                count: 2,
                interval: 1,
                timeout: 15,
            },
            DiagnosticMode::Deep => Self {
                count: 3,
                interval: 1,
                timeout: 30,
            },
        }
    }

    /// 获取总预计执行时间（秒）
    pub fn estimated_duration(&self) -> u64 {
        (self.count as u64 - 1) * self.interval as u64 + self.timeout
    }

    /// 验证采样配置的有效性
    pub fn is_valid(&self) -> bool {
        self.count >= 1
            && self.count <= 10
            && self.interval >= 1
            && self.interval <= 5
            && self.timeout >= 5
            && self.timeout <= 60
    }
}

/// 系统问题类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SystemIssueType {
    /// CPU使用率过高
    HighCpuUsage,
    /// 内存使用率过高
    HighMemoryUsage,
    /// I/O等待时间过长
    HighIoWait,
    /// 磁盘空间不足
    LowDiskSpace,
    /// 网络连接异常
    NetworkIssues,
    /// 进程异常
    ProcessAnomalies,
    /// 其他问题
    Other(String),
}

/// 系统问题严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
pub enum IssueSeverity {
    /// 信息级别，一般不需要处理
    Info,
    /// 警告级别，建议关注
    Warning,
    /// 错误级别，需要处理
    Error,
    /// 严重错误级别，需要立即处理
    Critical,
}

/// 系统问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemIssue {
    /// 问题类型
    pub issue_type: SystemIssueType,

    /// 问题描述
    pub description: String,

    /// 严重程度
    pub severity: IssueSeverity,

    /// 相关的数值或指标
    pub metrics: HashMap<String, serde_json::Value>,

    /// 建议的解决方案
    pub recommendations: Vec<String>,
}

/// 系统建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// 建议标题
    pub title: String,

    /// 建议描述
    pub description: String,

    /// 优先级（1-5，5为最高）
    pub priority: u8,

    /// 建议的具体操作步骤
    pub steps: Vec<String>,

    /// 预期效果
    pub expected_outcome: String,
}

/// 诊断报告数据结构
///
/// 包含系统诊断的完整结果，包括系统信息、问题识别和建议。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    /// 报告生成时间
    pub timestamp: DateTime<Utc>,

    /// 使用的诊断模式
    pub mode: DiagnosticMode,

    /// 系统基础信息
    pub system_info: SystemInfo,

    /// 性能指标数据
    pub performance_metrics: PerformanceMetrics,

    /// 识别出的问题列表
    pub issues_identified: Vec<SystemIssue>,

    /// 提供的建议列表
    pub recommendations: Vec<Recommendation>,

    /// 执行摘要
    pub execution_summary: ExecutionSummary,
}

/// 系统基础信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// 系统运行时间
    pub uptime: String,

    /// 系统负载
    pub load_average: Vec<f64>,

    /// 系统主机名
    pub hostname: Option<String>,

    /// 操作系统信息
    pub os_info: Option<String>,

    /// CPU核心数
    pub cpu_cores: Option<u32>,
}

/// 性能指标数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    /// CPU使用率相关指标
    pub cpu_metrics: CpuMetrics,

    /// 内存使用相关指标
    pub memory_metrics: MemoryMetrics,

    /// I/O性能相关指标
    pub io_metrics: Option<IoMetrics>,

    /// 网络相关指标
    pub network_metrics: Option<NetworkMetrics>,

    /// 进程相关指标
    pub process_metrics: Option<ProcessMetrics>,
}

/// CPU性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    /// 当前CPU使用率 (%)
    pub usage_percent: f64,

    /// 系统CPU时间 (%)
    pub system_percent: f64,

    /// 用户CPU时间 (%)
    pub user_percent: f64,

    /// 空闲CPU时间 (%)
    pub idle_percent: f64,
}

/// 内存性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// 总内存量 (bytes)
    pub total_memory: u64,

    /// 已使用内存 (bytes)
    pub used_memory: u64,

    /// 空闲内存 (bytes)
    pub free_memory: u64,

    /// 内存使用率 (%)
    pub usage_percent: f64,

    /// 缓存内存 (bytes)
    pub cached_memory: Option<u64>,

    /// 交换分区使用情况
    pub swap_info: Option<SwapInfo>,
}

/// 交换分区信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapInfo {
    /// 总交换分区大小 (bytes)
    pub total_swap: u64,

    /// 已使用交换分区 (bytes)
    pub used_swap: u64,

    /// 交换分区使用率 (%)
    pub usage_percent: f64,
}

/// I/O性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoMetrics {
    /// 磁盘读取速度 (bytes/sec)
    pub read_bytes_per_sec: f64,

    /// 磁盘写入速度 (bytes/sec)
    pub write_bytes_per_sec: f64,

    /// I/O等待时间 (%)
    pub io_wait_percent: f64,

    /// 磁盘使用率 (%)
    pub disk_usage_percent: f64,
}

/// 网络性能指标
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkMetrics {
    /// TCP连接数
    pub tcp_connections: u32,

    /// UDP连接数
    pub udp_connections: u32,

    /// 监听端口数
    pub listening_ports: u32,

    /// 已建立连接数
    pub established_connections: u32,

    /// 网络错误统计
    pub network_errors: u32,
}

/// 进程性能指标
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessMetrics {
    /// 总进程数
    pub total_processes: u32,

    /// 运行中进程数
    pub running_processes: u32,

    /// 睡眠进程数
    pub sleeping_processes: u32,

    /// 僵尸进程数
    pub zombie_processes: u32,

    /// 高CPU使用率进程数
    pub high_cpu_processes: u32,

    /// 高内存使用率进程数
    pub high_memory_processes: u32,
}

/// 执行摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    /// 总执行时间（秒）
    pub total_duration_seconds: f64,

    /// 执行的诊断函数列表
    pub executed_functions: Vec<String>,

    /// 成功执行的函数数量
    pub successful_functions: usize,

    /// 失败执行的函数数量
    pub failed_functions: usize,

    /// 诊断模式
    pub diagnostic_mode: String,

    /// 执行状态
    pub status: String,
}

impl DiagnosticReport {
    /// 创建新的诊断报告
    pub fn new(mode: DiagnosticMode) -> Self {
        Self {
            timestamp: Utc::now(),
            mode,
            system_info: SystemInfo::default(),
            performance_metrics: PerformanceMetrics::default(),
            issues_identified: Vec::new(),
            recommendations: Vec::new(),
            execution_summary: ExecutionSummary::new(mode.to_string()),
        }
    }

    /// 添加系统问题
    pub fn add_issue(&mut self, issue: SystemIssue) {
        self.issues_identified.push(issue);
    }

    /// 添加建议
    pub fn add_recommendation(&mut self, recommendation: Recommendation) {
        self.recommendations.push(recommendation);
    }

    /// 获取问题严重程度统计
    pub fn issue_severity_summary(&self) -> HashMap<IssueSeverity, usize> {
        let mut summary = HashMap::new();
        for issue in &self.issues_identified {
            *summary.entry(issue.severity.clone()).or_insert(0) += 1;
        }
        summary
    }

    /// 检查是否存在严重问题
    pub fn has_critical_issues(&self) -> bool {
        self.issues_identified
            .iter()
            .any(|issue| issue.severity == IssueSeverity::Critical)
    }

    /// 生成报告摘要
    pub fn summary(&self) -> String {
        format!(
            "Diagnostic Report ({}) - Mode: {}, Issues: {}, Recommendations: {}, Duration: {:.2}s",
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            self.mode.description(),
            self.issues_identified.len(),
            self.recommendations.len(),
            self.execution_summary.total_duration_seconds
        )
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            uptime: String::new(),
            load_average: vec![0.0, 0.0, 0.0],
            hostname: None,
            os_info: None,
            cpu_cores: None,
        }
    }
}

impl Default for CpuMetrics {
    fn default() -> Self {
        Self {
            usage_percent: 0.0,
            system_percent: 0.0,
            user_percent: 0.0,
            idle_percent: 100.0,
        }
    }
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self {
            total_memory: 0,
            used_memory: 0,
            free_memory: 0,
            usage_percent: 0.0,
            cached_memory: None,
            swap_info: None,
        }
    }
}

impl Default for IoMetrics {
    fn default() -> Self {
        Self {
            read_bytes_per_sec: 0.0,
            write_bytes_per_sec: 0.0,
            io_wait_percent: 0.0,
            disk_usage_percent: 0.0,
        }
    }
}

impl ExecutionSummary {
    fn new(mode: String) -> Self {
        Self {
            total_duration_seconds: 0.0,
            executed_functions: Vec::new(),
            successful_functions: 0,
            failed_functions: 0,
            diagnostic_mode: mode,
            status: "completed".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_mode_display() {
        assert_eq!(DiagnosticMode::Quick.to_string(), "quick");
        assert_eq!(DiagnosticMode::Standard.to_string(), "standard");
        assert_eq!(DiagnosticMode::Deep.to_string(), "deep");
    }

    #[test]
    fn test_diagnostic_mode_properties() {
        let quick = DiagnosticMode::Quick;
        assert_eq!(quick.description(), "快速系统扫描 (200ms内)");
        assert_eq!(quick.timeout_seconds(), 5);
        assert!(!quick.should_analyze_processes());
        assert!(!quick.should_analyze_io());

        let standard = DiagnosticMode::Standard;
        assert!(standard.should_analyze_processes());
        assert!(!standard.should_analyze_io());

        let deep = DiagnosticMode::Deep;
        assert!(deep.should_analyze_processes());
        assert!(deep.should_analyze_io());
        assert!(deep.should_analyze_network());
    }

    #[test]
    fn test_sampling_config() {
        let config = SamplingConfig::new(3, 2, 20);
        assert_eq!(config.count, 3);
        assert_eq!(config.interval, 2);
        assert_eq!(config.timeout, 20);
        assert_eq!(config.estimated_duration(), 24); // (3-1)*2 + 20 = 4 + 20 = 24
        assert!(config.is_valid());

        let adaptive = SamplingConfig::for_diagnostic_mode(&DiagnosticMode::Deep);
        assert_eq!(adaptive.count, 3);
        assert_eq!(adaptive.interval, 1);
        assert_eq!(adaptive.timeout, 30);
        assert_eq!(adaptive.estimated_duration(), 32); // (3-1)*1 + 30 = 2 + 30 = 32
    }

    #[test]
    fn test_diagnostic_report() {
        let mut report = DiagnosticReport::new(DiagnosticMode::Standard);

        // 添加测试问题
        let issue = SystemIssue {
            issue_type: SystemIssueType::HighCpuUsage,
            description: "CPU使用率过高".to_string(),
            severity: IssueSeverity::Warning,
            metrics: HashMap::new(),
            recommendations: vec!["检查高CPU进程".to_string()],
        };
        report.add_issue(issue);

        // 添加测试建议
        let recommendation = Recommendation {
            title: "优化CPU使用".to_string(),
            description: "建议检查和优化高CPU使用进程".to_string(),
            priority: 3,
            steps: vec!["使用top命令检查CPU使用".to_string()],
            expected_outcome: "CPU使用率降低到正常范围".to_string(),
        };
        report.add_recommendation(recommendation);

        assert_eq!(report.issues_identified.len(), 1);
        assert_eq!(report.recommendations.len(), 1);
        assert!(!report.has_critical_issues());

        let summary = report.summary();
        assert!(summary.contains("Diagnostic Report"));
        assert!(summary.contains("标准系统诊断 (1-2秒)")); // Mode description contains this
    }

    #[test]
    fn test_default_implementations() {
        let default_mode = DiagnosticMode::default();
        assert_eq!(default_mode, DiagnosticMode::Standard);

        let default_config = SamplingConfig::default();
        assert!(default_config.is_valid());

        let default_report = DiagnosticReport::new(DiagnosticMode::Quick);
        assert_eq!(default_report.mode, DiagnosticMode::Quick);
    }
}
