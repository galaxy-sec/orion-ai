use orion_error::{ToStructError, UvsLogicFrom};
use serde_json::json;

use crate::{
    AiResult, FunctionCall, FunctionDefinition, FunctionExecutor, FunctionParameter,
    FunctionResult, error::OrionAiReason,
};

use super::{execute_command_with_timeout, parse_function_arguments};

// 进程监控函数执行器
pub struct MonitorExecutor;

#[async_trait::async_trait]
impl FunctionExecutor for MonitorExecutor {
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        match function_call.function.name.as_str() {
            "sys-proc-top" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let sort_by = args
                    .get("sort_by")
                    .and_then(|v| v.as_str())
                    .unwrap_or("cpu");
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

                let mut command_args = vec!["aux"];

                // 添加排序参数
                match sort_by {
                    "cpu" => command_args.push("--sort=-%cpu"),
                    "memory" | "mem" => command_args.push("--sort=-%mem"),
                    _ => command_args.push("--sort=-%cpu"),
                }

                match execute_command_with_timeout("ps", &command_args, 15).await {
                    Ok(output) => {
                        let result = String::from_utf8_lossy(&output.stdout).to_string();
                        let lines: Vec<String> = result.lines().map(|s| s.to_string()).collect();

                        // 应用limit限制
                        let limited_lines = if lines.len() > limit + 1 {
                            // +1 for header
                            lines[..limit + 1].to_vec()
                        } else {
                            lines
                        };

                        Ok(FunctionResult {
                            name: "sys-proc-top".to_string(),
                            result: json!({
                                "processes": limited_lines,
                                "sort_by": sort_by,
                                "limit": limit,
                                "success": output.status.success()
                            }),
                            error: if output.status.success() {
                                None
                            } else {
                                Some(String::from_utf8_lossy(&output.stderr).to_string())
                            },
                        })
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "sys-proc-top".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to get process top info: {}", e)),
                    }),
                }
            }

            "sys-proc-stats" => {
                match execute_command_with_timeout("ps", &["axo", "pid,stat,comm"], 10).await {
                    Ok(output) => {
                        let result = String::from_utf8_lossy(&output.stdout).to_string();
                        let lines: Vec<String> = result.lines().map(|s| s.to_string()).collect();

                        // 统计进程状态
                        let mut stats = std::collections::HashMap::new();
                        for line in lines.iter().skip(1) {
                            // skip header
                            if let Some(status_char) = line.chars().next() {
                                *stats.entry(status_char.to_string()).or_insert(0) += 1;
                            }
                        }

                        Ok(FunctionResult {
                            name: "sys-proc-stats".to_string(),
                            result: json!({
                                "process_lines": lines,
                                "status_stats": stats,
                                "total_processes": lines.len().saturating_sub(1), // exclude header
                                "success": output.status.success()
                            }),
                            error: if output.status.success() {
                                None
                            } else {
                                Some(String::from_utf8_lossy(&output.stderr).to_string())
                            },
                        })
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "sys-proc-stats".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to get process stats: {}", e)),
                    }),
                }
            }

            _ => Err(OrionAiReason::from_logic("Unknown monitor function".to_string()).to_err()),
        }
    }

    fn supported_functions(&self) -> Vec<String> {
        vec!["sys-proc-top".to_string(), "sys-proc-stats".to_string()]
    }

    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
        create_monitor_functions()
            .into_iter()
            .find(|f| f.name == function_name)
    }
}

pub fn create_monitor_functions() -> Vec<FunctionDefinition> {
    vec![
        FunctionDefinition {
            name: "sys-proc-top".to_string(),
            description: "显示高资源消耗进程列表，可按CPU或内存排序".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "sort_by".to_string(),
                    description: "排序方式: 'cpu' 或 'memory'，默认为 'cpu'".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                },
                FunctionParameter {
                    name: "limit".to_string(),
                    description: "显示进程数量限制，默认为10".to_string(),
                    r#type: "number".to_string(),
                    required: false,
                },
            ],
        },
        FunctionDefinition {
            name: "sys-proc-stats".to_string(),
            description: "显示进程统计信息，包括进程状态分布".to_string(),
            parameters: vec![],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitor_executor() {
        let executor = MonitorExecutor;

        // 测试支持的函数列表
        let functions = executor.supported_functions();
        assert!(functions.contains(&"sys-proc-top".to_string()));
        assert!(functions.contains(&"sys-proc-stats".to_string()));
    }

    #[tokio::test]
    async fn test_create_monitor_functions() {
        let functions = create_monitor_functions();
        assert_eq!(functions.len(), 2);

        // 验证 proc-top 函数定义
        let proc_top_func = functions.iter().find(|f| f.name == "sys-proc-top").unwrap();
        assert_eq!(
            proc_top_func.description,
            "显示高资源消耗进程列表，可按CPU或内存排序"
        );
        assert_eq!(proc_top_func.parameters.len(), 2);
        assert_eq!(proc_top_func.parameters[0].name, "sort_by");
        assert!(!proc_top_func.parameters[0].required);
        assert_eq!(proc_top_func.parameters[1].name, "limit");
        assert!(!proc_top_func.parameters[1].required);

        // 验证 proc-stats 函数定义
        let proc_stats_func = functions
            .iter()
            .find(|f| f.name == "sys-proc-stats")
            .unwrap();
        assert_eq!(
            proc_stats_func.description,
            "显示进程统计信息，包括进程状态分布"
        );
        assert_eq!(proc_stats_func.parameters.len(), 0);
    }
}
