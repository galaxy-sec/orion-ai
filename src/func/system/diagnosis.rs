use orion_error::{ToStructError, UvsLogicFrom};
use serde_json::json;

use crate::{
    AiResult, FunctionCall, FunctionDefinition, FunctionExecutor, FunctionParameter,
    FunctionResult, error::OrionAiReason,
};

use super::{execute_command_with_timeout, parse_function_arguments};

// 系统诊断函数执行器
pub struct DiagnosisExecutor;

#[async_trait::async_trait]
impl FunctionExecutor for DiagnosisExecutor {
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        match function_call.function.name.as_str() {
            "sys-uptime" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let detailed = args
                    .get("detailed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let command_args = if detailed { vec!["-a"] } else { vec![] };

                match execute_command_with_timeout("uptime", &command_args, 5).await {
                    Ok(output) => {
                        let result = String::from_utf8_lossy(&output.stdout).to_string();
                        Ok(FunctionResult {
                            name: "sys-uptime".to_string(),
                            result: json!({
                                "output": result.trim(),
                                "detailed": detailed,
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
                        name: "sys-uptime".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to get uptime info: {}", e)),
                    }),
                }
            }

            "sys-meminfo" => match execute_command_with_timeout("vm_stat", &[], 5).await {
                Ok(output) => {
                    let result = String::from_utf8_lossy(&output.stdout).to_string();
                    let lines: Vec<String> = result.lines().map(|s| s.to_string()).collect();

                    Ok(FunctionResult {
                        name: "sys-meminfo".to_string(),
                        result: json!({
                            "memory_info": lines,
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
                    name: "sys-meminfo".to_string(),
                    result: serde_json::Value::Null,
                    error: Some(format!("Failed to get memory info: {}", e)),
                }),
            },

            _ => Err(OrionAiReason::from_logic("Unknown diagnosis function".to_string()).to_err()),
        }
    }

    fn supported_functions(&self) -> Vec<String> {
        vec!["sys-uptime".to_string(), "sys-meminfo".to_string()]
    }

    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
        create_diagnosis_functions()
            .into_iter()
            .find(|f| f.name == function_name)
    }
}

pub fn create_diagnosis_functions() -> Vec<FunctionDefinition> {
    vec![
        FunctionDefinition {
            name: "sys-uptime".to_string(),
            description: "显示系统运行时间和负载".to_string(),
            parameters: vec![FunctionParameter {
                name: "detailed".to_string(),
                description: "是否显示详细信息，默认为false".to_string(),
                r#type: "boolean".to_string(),
                required: false,
            }],
        },
        FunctionDefinition {
            name: "sys-meminfo".to_string(),
            description: "显示内存使用详细信息".to_string(),
            parameters: vec![],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_diagnosis_executor() {
        let executor = DiagnosisExecutor;

        // 测试支持的函数列表
        let functions = executor.supported_functions();
        assert!(functions.contains(&"sys-uptime".to_string()));
        assert!(functions.contains(&"sys-meminfo".to_string()));
    }

    #[tokio::test]
    async fn test_create_diagnosis_functions() {
        let functions = create_diagnosis_functions();
        assert_eq!(functions.len(), 2);

        // 验证 uptime 函数定义
        let uptime_func = functions.iter().find(|f| f.name == "sys-uptime").unwrap();
        assert_eq!(uptime_func.description, "显示系统运行时间和负载");
        assert_eq!(uptime_func.parameters.len(), 1);
        assert_eq!(uptime_func.parameters[0].name, "detailed");
        assert!(!uptime_func.parameters[0].required);

        // 验证 meminfo 函数定义
        let meminfo_func = functions.iter().find(|f| f.name == "sys-meminfo").unwrap();
        assert_eq!(meminfo_func.description, "显示内存使用详细信息");
        assert_eq!(meminfo_func.parameters.len(), 0);
    }
}
