//! 系统诊断相关类型定义
//!
//! 这个模块包含了系统诊断工具使用的核心类型定义，
//! 包括诊断模式、采样配置和诊断报告等。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 诊断深度级别
///
/// 定义系统诊断的深度级别，用于控制诊断的详细程度。
/// 不绑定具体时间约束，而是描述相对的诊断深度。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum DiagnosticDepth {
    /// 基础级别
    ///
    /// 执行最基本的系统健康检查：
    /// - 系统基本信息
    /// - 基础资源使用情况
    #[default]
    Basic,

    /// 标准级别
    ///
    /// 执行中等深度的系统分析：
    /// - 所有基础级别检查
    /// - 进程分析
    /// - 详细资源统计
    Standard,

    /// 深度级别
    ///
    /// 执行全面的系统诊断：
    /// - 所有标准级别检查
    /// - I/O性能分析
    /// - 网络连接分析
    /// - 综合性能评估
    Advanced,

    /// 自定义级别
    ///
    /// 允许用户自定义诊断配置，
    /// 通过 DiagnosticConfig 灵活控制诊断行为
    Custom(DiagnosticConfig),
}

impl std::fmt::Display for DiagnosticDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticDepth::Basic => write!(f, "basic"),
            DiagnosticDepth::Standard => write!(f, "standard"),
            DiagnosticDepth::Advanced => write!(f, "advanced"),
            DiagnosticDepth::Custom(_) => write!(f, "custom"),
        }
    }
}

/// 诊断配置结构
///
/// 提供灵活的诊断配置选项，不硬编码时间约束，
/// 允许用户根据具体需求调整诊断行为。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticConfig {
    /// 基础信息检查
    pub check_basic_info: bool,

    /// 进程分析
    pub check_processes: bool,

    /// I/O性能分析
    pub check_io_performance: bool,

    /// 网络分析
    pub check_network: bool,

    /// 采样次数
    pub sampling_count: u32,

    /// 采样间隔（秒）
    pub sampling_interval: u32,

    /// 超时时间（秒）
    pub timeout_seconds: u64,
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        Self {
            check_basic_info: true,
            check_processes: true,
            check_io_performance: false,
            check_network: false,
            sampling_count: 2,
            sampling_interval: 1,
            timeout_seconds: 15,
        }
    }
}

impl DiagnosticConfig {
    /// 创建基础配置
    pub fn basic() -> Self {
        Self {
            check_basic_info: true,
            check_processes: false,
            check_io_performance: false,
            check_network: false,
            sampling_count: 1,
            sampling_interval: 1,
            timeout_seconds: 5,
        }
    }

    /// 创建标准配置
    pub fn standard() -> Self {
        Self {
            check_basic_info: true,
            check_processes: true,
            check_io_performance: false,
            check_network: false,
            sampling_count: 2,
            sampling_interval: 1,
            timeout_seconds: 15,
        }
    }

    /// 创建高级配置
    pub fn advanced() -> Self {
        Self {
            check_basic_info: true,
            check_processes: true,
            check_io_performance: true,
            check_network: true,
            sampling_count: 3,
            sampling_interval: 1,
            timeout_seconds: 30,
        }
    }

    /// 验证配置的有效性
    pub fn is_valid(&self) -> bool {
        self.sampling_count >= 1
            && self.sampling_count <= 10
            && self.sampling_interval >= 1
            && self.sampling_interval <= 5
            && self.timeout_seconds >= 5
            && self.timeout_seconds <= 60
    }

    /// 获取预计执行时间（秒）
    pub fn estimated_duration(&self) -> u64 {
        (self.sampling_count as u64 - 1) * self.sampling_interval as u64 + self.timeout_seconds
    }
}

impl DiagnosticDepth {
    /// 获取诊断深度级别的描述信息
    pub fn description(&self) -> &'static str {
        match self {
            DiagnosticDepth::Basic => "基础诊断级别",
            DiagnosticDepth::Standard => "标准诊断级别",
            DiagnosticDepth::Advanced => "高级诊断级别",
            DiagnosticDepth::Custom(_) => "自定义诊断级别",
        }
    }

    /// 获取对应的诊断配置
    pub fn to_config(&self) -> DiagnosticConfig {
        match self {
            DiagnosticDepth::Basic => DiagnosticConfig::basic(),
            DiagnosticDepth::Standard => DiagnosticConfig::standard(),
            DiagnosticDepth::Advanced => DiagnosticConfig::advanced(),
            DiagnosticDepth::Custom(config) => config.clone(),
        }
    }

    /// 从配置创建自定义深度级别
    pub fn custom(config: DiagnosticConfig) -> Self {
        DiagnosticDepth::Custom(config)
    }

    /// 检查是否应该执行进程分析
    pub fn should_analyze_processes(&self) -> bool {
        match self {
            DiagnosticDepth::Basic => false,
            DiagnosticDepth::Standard => true,
            DiagnosticDepth::Advanced => true,
            DiagnosticDepth::Custom(config) => config.check_processes,
        }
    }

    /// 检查是否应该执行I/O分析
    pub fn should_analyze_io(&self) -> bool {
        match self {
            DiagnosticDepth::Basic => false,
            DiagnosticDepth::Standard => false,
            DiagnosticDepth::Advanced => true,
            DiagnosticDepth::Custom(config) => config.check_io_performance,
        }
    }

    /// 检查是否应该执行网络分析
    pub fn should_analyze_network(&self) -> bool {
        match self {
            DiagnosticDepth::Basic => false,
            DiagnosticDepth::Standard => false,
            DiagnosticDepth::Advanced => true,
            DiagnosticDepth::Custom(config) => config.check_network,
        }
    }

    /// 获取配置的超时时间
    pub fn timeout_seconds(&self) -> u64 {
        self.to_config().timeout_seconds
    }

    /// 获取配置的采样信息
    pub fn sampling_info(&self) -> (u32, u32) {
        let config = self.to_config();
        (config.sampling_count, config.sampling_interval)
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

    /// 根据诊断深度生成自适应采样配置
    pub fn for_diagnostic_depth(depth: &DiagnosticDepth) -> Self {
        let config = depth.to_config();
        Self {
            count: config.sampling_count,
            interval: config.sampling_interval,
            timeout: config.timeout_seconds,
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

    /// 使用的诊断深度
    pub mode: DiagnosticDepth,

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
    pub fn new(depth: DiagnosticDepth) -> Self {
        let mode_description = depth.description().to_string();
        Self {
            timestamp: Utc::now(),
            mode: depth,
            system_info: SystemInfo::default(),
            performance_metrics: PerformanceMetrics::default(),
            issues_identified: Vec::new(),
            recommendations: Vec::new(),
            execution_summary: ExecutionSummary::new(mode_description),
        }
    }

    /// 使用自定义配置创建诊断报告
    pub fn with_config(config: DiagnosticConfig) -> Self {
        Self::new(DiagnosticDepth::Custom(config))
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

    /// 生成格式化的详细报告
    pub fn formatted_report(&self) -> String {
        let mut report = String::new();
        
        // 报告标题
        report.push_str("=== 系统诊断报告 ===\n");
        report.push_str(&format!("时间: {}\n", self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        report.push_str(&format!("诊断模式: {}\n\n", self.mode.description()));
        
        // 系统信息
        report.push_str("--- 系统信息 ---\n");
        report.push_str(&format!("运行时间: {}\n", self.system_info.uptime));
        if let Some(hostname) = &self.system_info.hostname {
            report.push_str(&format!("主机名: {}\n", hostname));
        }
        if let Some(os_info) = &self.system_info.os_info {
            report.push_str(&format!("操作系统: {}\n", os_info));
        }
        if let Some(cpu_cores) = self.system_info.cpu_cores {
            report.push_str(&format!("CPU核心数: {}\n", cpu_cores));
        }
        report.push('\n');
        
        // 性能指标
        report.push_str("--- 性能指标 ---\n");
        report.push_str(&format!("CPU使用率: {:.1}%\n", self.performance_metrics.cpu_metrics.usage_percent));
        report.push_str(&format!("内存使用率: {:.1}%\n", self.performance_metrics.memory_metrics.usage_percent));
        if let Some(io_metrics) = &self.performance_metrics.io_metrics {
            report.push_str(&format!("I/O等待: {:.1}%\n", io_metrics.io_wait_percent));
            report.push_str(&format!("磁盘使用率: {:.1}%\n", io_metrics.disk_usage_percent));
        }
        if let Some(network_metrics) = &self.performance_metrics.network_metrics {
            report.push_str(&format!("TCP连接数: {}\n", network_metrics.tcp_connections));
            report.push_str(&format!("UDP连接数: {}\n", network_metrics.udp_connections));
        }
        report.push('\n');
        
        // 识别的问题
        if !self.issues_identified.is_empty() {
            report.push_str("--- 识别的问题 ---\n");
            for (i, issue) in self.issues_identified.iter().enumerate() {
                report.push_str(&format!("{}. {}\n", i + 1, issue.description));
                report.push_str(&format!("   严重程度: {}\n", issue.severity.severity_to_chinese()));
                if !issue.recommendations.is_empty() {
                    report.push_str("   建议: ");
                    for (j, rec) in issue.recommendations.iter().enumerate() {
                        if j > 0 {
                            report.push_str(", ");
                        }
                        report.push_str(rec);
                    }
                    report.push('\n');
                }
                report.push('\n');
            }
        } else {
            report.push_str("--- 识别的问题 ---\n");
            report.push_str("未发现系统问题\n\n");
        }
        
        // 建议
        if !self.recommendations.is_empty() {
            report.push_str("--- 建议 ---\n");
            for (i, rec) in self.recommendations.iter().enumerate() {
                report.push_str(&format!("{}. [优先级: {}] {}\n", i + 1, rec.priority, rec.title));
                report.push_str(&format!("   {}\n", rec.description));
                if !rec.steps.is_empty() {
                    report.push_str("   操作步骤:\n");
                    for step in &rec.steps {
                        report.push_str(&format!("   - {}\n", step));
                    }
                }
                report.push_str(&format!("   预期效果: {}\n\n", rec.expected_outcome));
            }
        }
        
        // 执行摘要
        report.push_str("--- 执行摘要 ---\n");
        report.push_str(&format!("总执行时间: {:.2}秒\n", self.execution_summary.total_duration_seconds));
        report.push_str(&format!("成功执行函数: {}\n", self.execution_summary.successful_functions));
        report.push_str(&format!("失败执行函数: {}\n", self.execution_summary.failed_functions));
        report.push_str(&format!("执行状态: {}\n", self.execution_summary.status));
        
        report
    }
}

impl IssueSeverity {
    /// 将严重程度转换为中文描述
    pub fn to_chinese(&self) -> &'static str {
        match self {
            IssueSeverity::Info => "信息",
            IssueSeverity::Warning => "警告",
            IssueSeverity::Error => "错误",
            IssueSeverity::Critical => "严重",
        }
    }
    
    /// 将严重程度转换为中文描述（用于格式化输出）
    pub fn severity_to_chinese(&self) -> &'static str {
        self.to_chinese()
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
    fn test_diagnostic_depth_config() {
        let basic_config = DiagnosticConfig::basic();
        assert!(basic_config.check_basic_info);
        assert!(!basic_config.check_processes);
        assert_eq!(basic_config.timeout_seconds, 5);
        assert!(basic_config.is_valid());

        let standard_config = DiagnosticConfig::standard();
        assert!(standard_config.check_basic_info);
        assert!(standard_config.check_processes);
        assert!(!standard_config.check_io_performance);
        assert_eq!(standard_config.timeout_seconds, 15);
        assert!(standard_config.is_valid());

        let advanced_config = DiagnosticConfig::advanced();
        assert!(advanced_config.check_basic_info);
        assert!(advanced_config.check_processes);
        assert!(advanced_config.check_io_performance);
        assert!(advanced_config.check_network);
        assert_eq!(advanced_config.timeout_seconds, 30);
        assert!(advanced_config.is_valid());
    }

    #[test]
    fn test_diagnostic_depth_properties() {
        let basic = DiagnosticDepth::Basic;
        assert_eq!(basic.description(), "基础诊断级别");
        assert!(!basic.should_analyze_processes());
        assert!(!basic.should_analyze_io());
        assert_eq!(basic.timeout_seconds(), 5);

        let standard = DiagnosticDepth::Standard;
        assert!(standard.should_analyze_processes());
        assert!(!standard.should_analyze_io());
        assert_eq!(standard.timeout_seconds(), 15);

        let advanced = DiagnosticDepth::Advanced;
        assert!(advanced.should_analyze_processes());
        assert!(advanced.should_analyze_io());
        assert!(advanced.should_analyze_network());
        assert_eq!(advanced.timeout_seconds(), 30);

        let custom_config = DiagnosticConfig {
            check_basic_info: true,
            check_processes: true,
            check_io_performance: true,
            check_network: false,
            sampling_count: 4,
            sampling_interval: 2,
            timeout_seconds: 25,
        };
        let custom = DiagnosticDepth::custom(custom_config);
        assert!(custom.should_analyze_processes());
        assert!(custom.should_analyze_io());
        assert!(!custom.should_analyze_network());
        assert_eq!(custom.timeout_seconds(), 25);
    }

    #[test]
    fn test_sampling_config() {
        let config = SamplingConfig::new(3, 2, 20);
        assert_eq!(config.count, 3);
        assert_eq!(config.interval, 2);
        assert_eq!(config.timeout, 20);
        assert_eq!(config.estimated_duration(), 24); // (3-1)*2 + 20
        assert!(config.is_valid());

        // 测试从诊断深度创建采样配置
        let depth = DiagnosticDepth::Advanced;
        let (count, interval) = depth.sampling_info();
        assert_eq!(count, 3);
        assert_eq!(interval, 1);
    }

    #[test]
    fn test_diagnostic_report() {
        let mut report = DiagnosticReport::new(DiagnosticDepth::Standard);

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
        assert!(summary.contains("标准诊断级别"));
    }

    #[test]
    fn test_diagnostic_report_formatted_report() {
        let mut report = DiagnosticReport::new(DiagnosticDepth::Standard);

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

        let formatted = report.formatted_report();
        
        // 验证报告包含必要部分
        assert!(formatted.contains("系统诊断报告"));
        assert!(formatted.contains("CPU使用率过高"));
        assert!(formatted.contains("优化CPU使用"));
        assert!(formatted.contains("严重程度: 警告"));
        assert!(formatted.contains("优先级: 3"));
    }

    #[test]
    fn test_issue_severity_to_chinese() {
        assert_eq!(IssueSeverity::Critical.to_chinese(), "严重");
        assert_eq!(IssueSeverity::Warning.to_chinese(), "警告");
        assert_eq!(IssueSeverity::Info.to_chinese(), "信息");
        
        // 测试静态方法
        assert_eq!(IssueSeverity::severity_to_chinese(&IssueSeverity::Critical), "严重");
        assert_eq!(IssueSeverity::severity_to_chinese(&IssueSeverity::Warning), "警告");
        assert_eq!(IssueSeverity::severity_to_chinese(&IssueSeverity::Info), "信息");
    }

    #[test]
    fn test_default_implementations() {
        let default_depth = DiagnosticDepth::default();
        assert_eq!(default_depth, DiagnosticDepth::Basic);

        let default_config = DiagnosticConfig::default();
        assert!(default_config.is_valid());

        // 测试自定义配置
        let custom_config = DiagnosticConfig {
            check_basic_info: true,
            check_processes: false,
            check_io_performance: true,
            check_network: false,
            sampling_count: 1,
            sampling_interval: 1,
            timeout_seconds: 10,
        };
        let custom_report = DiagnosticReport::with_config(custom_config);
        assert!(matches!(custom_report.mode, DiagnosticDepth::Custom(_)));
    }
}
