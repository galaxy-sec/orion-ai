//! AI执行单元模块
//!
//! 此模块提供了 AiExecUnit 和相关类型，用于封装AI执行所需的核心组件，
//! 提供统一的执行接口。

mod builder;
mod diagnosis;
mod unit;

// 重新导出主要类型
pub use builder::AiExecUnitBuilder;
pub use diagnosis::{
    DiagnosisError, DiagnosticExecutor, ProgressiveDiagnosis, deep_analysis, progressive_diagnosis,
    quick_health_check, standard_diagnosis,
};
pub use unit::AiExecUnit;
