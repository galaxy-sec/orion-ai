use orion_error::{ToStructError, UvsLogicFrom};
use serde_json::json;

use crate::{
    AiResult, FunctionCall, FunctionDefinition, FunctionExecutor, FunctionParameter,
    FunctionResult, error::OrionAiReason,
};

use super::{execute_command_with_timeout, parse_function_arguments, detect_platform, get_platform_specific_command, platform_name, CommandType, Platform};

/// 解析 macOS vm_stat 命令输出
pub fn parse_vmstat_output(output: &str) -> serde_json::Value {
    let mut memory_info = serde_json::Map::new();
    let mut page_size = 4096; // 默认页面大小
    
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        
        // 解析页面大小
        if trimmed.starts_with("Mach Virtual Memory Statistics:") {
            continue;
        }
        
        if trimmed.starts_with("page size of") {
            if let Some(size_str) = trimmed.split_whitespace().nth(3) {
                if let Ok(size) = size_str.parse::<u64>() {
                    page_size = size;
                }
            }
            continue;
        }
        
        // 解析内存统计信息
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim().to_string();
            let value_str = trimmed[colon_pos + 1..].trim();
            
            // 移除点号并转换为数字
            let clean_value = value_str.replace('.', "");
            if let Ok(value) = clean_value.parse::<u64>() {
                // 将页面数转换为字节
                let bytes = value * page_size;
                memory_info.insert(key, json!(bytes));
            }
        }
    }
    
    // 计算总内存和使用内存
    if let (Some(free), Some(active), Some(inactive), Some(wire)) = (
        memory_info.get("Pages free").and_then(|v| v.as_u64()),
        memory_info.get("Pages active").and_then(|v| v.as_u64()),
        memory_info.get("Pages inactive").and_then(|v| v.as_u64()),
        memory_info.get("Pages wired").and_then(|v| v.as_u64()),
    ) {
        let total = free + active + inactive + wire;
        let used = active + inactive + wire;
        let free_percent = (free as f64 / total as f64) * 100.0;
        let used_percent = (used as f64 / total as f64) * 100.0;
        
        memory_info.insert("total_memory".to_string(), json!(total));
        memory_info.insert("used_memory".to_string(), json!(used));
        memory_info.insert("free_memory".to_string(), json!(free));
        memory_info.insert("free_percent".to_string(), json!(format!("{:.2}%", free_percent)));
        memory_info.insert("used_percent".to_string(), json!(format!("{:.2}%", used_percent)));
    }
    
    serde_json::Value::Object(memory_info)
}

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

                // 检测当前平台并获取平台特定命令
                let platform = detect_platform();
                let (command, default_args) = get_platform_specific_command(CommandType::Uptime, &platform);
                
                // 构建命令参数
                let mut command_args = default_args.clone();
                
                // 根据detailed参数调整命令
                if detailed {
                    match platform {
                        Platform::MacOS => {
                            command_args.push("-v");
                        },
                        Platform::Linux => {
                            command_args.push("-p");
                        },
                        Platform::Windows => {
                            // Windows uptime命令不支持详细参数
                        },
                        Platform::Unknown => {
                            // 未知平台，尝试添加详细参数
                            command_args.push("-v");
                        }
                    }
                }

                match execute_command_with_timeout(&command, &command_args.iter().map(|s| s.as_ref()).collect::<Vec<&str>>(), 5).await {
                    Ok(output) => {
                        let result = String::from_utf8_lossy(&output.stdout).to_string();
                        Ok(FunctionResult {
                            name: "sys-uptime".to_string(),
                            result: json!({
                                "output": result.trim(),
                                "detailed": detailed,
                                "success": output.status.success(),
                                "platform": platform_name(&platform),
                                "command": format!("{} {}", command, command_args.join(" "))
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

            "sys-meminfo" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let detailed = args
                    .get("detailed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                // 检测当前平台并获取平台特定命令
                let platform = detect_platform();
                let (command, default_args) = get_platform_specific_command(CommandType::MemInfo, &platform);
                
                // 构建命令参数
                let mut command_args = default_args.clone();
                
                // 根据detailed参数调整命令
                if detailed {
                    match platform {
                        Platform::MacOS => {
                            command_args.push("-w");
                        },
                        Platform::Linux => {
                            command_args.push("-h");
                        },
                        Platform::Windows => {
                            // Windows meminfo命令不支持详细参数
                        },
                        Platform::Unknown => {
                            // 未知平台，尝试添加详细参数
                            command_args.push("-h");
                        }
                    }
                }

                match execute_command_with_timeout(&command, &command_args.iter().map(|s| s.as_ref()).collect::<Vec<&str>>(), 5).await {
                    Ok(output) => {
                        let result = String::from_utf8_lossy(&output.stdout).to_string();
                        
                        // 根据平台解析输出
                        let parsed_result = match platform {
                            Platform::MacOS => parse_vmstat_output(&result),
                            Platform::Linux => {
                                // 对于 Linux，简单返回原始输出，因为 free 命令的输出已经比较友好
                                let lines: Vec<String> = result.lines().map(|s| s.to_string()).collect();
                                json!({
                                    "memory_info": lines,
                                    "platform": platform_name(&platform),
                                    "command": format!("{} {}", command, command_args.join(" "))
                                })
                            }
                            Platform::Windows => {
                                // 对于 Windows，简单返回原始输出
                                let lines: Vec<String> = result.lines().map(|s| s.to_string()).collect();
                                json!({
                                    "memory_info": lines,
                                    "platform": platform_name(&platform),
                                    "command": format!("{} {}", command, command_args.join(" "))
                                })
                            }
                            Platform::Unknown => {
                                json!({
                                    "raw_output": result.trim(),
                                    "platform": platform_name(&platform),
                                    "command": format!("{} {}", command, command_args.join(" "))
                                })
                            }
                        };
                        
                        Ok(FunctionResult {
                            name: "sys-meminfo".to_string(),
                            result: parsed_result,
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
                }
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
