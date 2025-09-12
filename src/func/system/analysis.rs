use orion_error::{ToStructError, UvsLogicFrom};
use serde_json::json;

use crate::{
    AiResult, FunctionCall, FunctionDefinition, FunctionExecutor, FunctionParameter,
    FunctionResult, error::OrionAiReason,
};

use super::{execute_command_with_timeout, parse_function_arguments, detect_platform, get_platform_specific_command, platform_name, CommandType, Platform};
use super::platform::parse_vmstat_output;

// 性能分析函数执行器
pub struct AnalysisExecutor;

#[async_trait::async_trait]
impl FunctionExecutor for AnalysisExecutor {
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        match function_call.function.name.as_str() {
            "sys-iostat" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(2) as u32;
                let interval = args.get("interval").and_then(|v| v.as_u64()).unwrap_or(1) as u32;

                // 限制采样次数防止超时
                let safe_count = count.clamp(1, 5);
                let safe_interval = interval.clamp(1, 3);

                // 检测当前平台
                let platform = detect_platform();
                let platform_name_str = platform_name(&platform);
                
                // 获取平台特定的iostat命令
                let (command, default_args) = get_platform_specific_command(CommandType::Iostat, &platform);
                
                // 构建命令参数
                let mut command_args = default_args.clone();
                
                // 根据平台和参数调整命令
                let interval_str = safe_interval.to_string();
                let count_str = safe_count.to_string();
                
                match platform {
                    Platform::MacOS => {
                        // macOS的iostat命令参数
                        command_args.push(&interval_str);
                        command_args.push(&count_str);
                    },
                    Platform::Linux => {
                        // Linux的iostat命令参数
                        command_args.push(&interval_str);
                        command_args.push(&count_str);
                    },
                    Platform::Windows => {
                        // Windows没有原生的iostat命令，使用typeperf替代
                        command_args.push(&interval_str);
                    },
                    Platform::Unknown => {
                        // 未知平台，尝试通用参数
                        command_args.push(&interval_str);
                        command_args.push(&count_str);
                    }
                }

                match execute_command_with_timeout(&command, &command_args, 20).await {
                    Ok(output) => {
                        let result = String::from_utf8_lossy(&output.stdout).to_string();
                        let lines: Vec<String> = result.lines().map(|s| s.to_string()).collect();

                        Ok(FunctionResult {
                            name: "sys-iostat".to_string(),
                            result: json!({
                                "iostat_output": lines,
                                "count": safe_count,
                                "interval": safe_interval,
                                "platform": platform_name_str,
                                "command": {
                                    "executable": command,
                                    "arguments": command_args
                                },
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
                        name: "sys-iostat".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to get I/O statistics: {}", e)),
                    }),
                }
            }

            "sys-netstat" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let show_tcp = args
                    .get("show_tcp")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let show_udp = args
                    .get("show_udp")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                // 检测当前平台
                let platform = detect_platform();
                let platform_name_str = platform_name(&platform);
                
                // 获取平台特定的netstat命令
                let (command, default_args) = get_platform_specific_command(CommandType::NetStat, &platform);
                
                // 构建命令参数
                let mut command_args = default_args.clone();
                
                // 根据平台和参数调整命令
                match platform {
                    Platform::MacOS => {
                        // macOS的netstat命令参数
                        if show_tcp && !show_udp {
                            command_args.push("-p");
                            command_args.push("tcp");
                        } else if show_udp && !show_tcp {
                            command_args.push("-p");
                            command_args.push("udp");
                        }
                    },
                    Platform::Linux => {
                        // Linux的netstat命令参数
                        if show_tcp && !show_udp {
                            command_args.push("-t");
                        } else if show_udp && !show_tcp {
                            command_args.push("-u");
                        }
                    },
                    Platform::Windows => {
                        // Windows的netstat命令参数
                        if show_tcp && !show_udp {
                            command_args.push("-p");
                            command_args.push("tcp");
                        } else if show_udp && !show_tcp {
                            command_args.push("-p");
                            command_args.push("udp");
                        }
                    },
                    Platform::Unknown => {
                        // 未知平台，使用默认参数
                        if show_tcp && !show_udp {
                            command_args.push("-p");
                            command_args.push("tcp");
                        } else if show_udp && !show_tcp {
                            command_args.push("-p");
                            command_args.push("udp");
                        }
                    }
                }

                match execute_command_with_timeout(&command, &command_args, 15).await {
                    Ok(output) => {
                        let result = String::from_utf8_lossy(&output.stdout).to_string();
                        let lines: Vec<String> = result.lines().map(|s| s.to_string()).collect();

                        // 平台特定的连接统计
                        let mut tcp_count = 0;
                        let mut udp_count = 0;
                        let mut listening_count = 0;
                        let mut established_count = 0;

                        for line in &lines {
                            match platform {
                                Platform::MacOS => {
                                    // macOS netstat输出解析
                                    if line.contains("tcp") {
                                        tcp_count += 1;
                                        if line.contains("LISTEN") {
                                            listening_count += 1;
                                        } else if line.contains("ESTABLISHED") {
                                            established_count += 1;
                                        }
                                    } else if line.contains("udp") {
                                        udp_count += 1;
                                    }
                                },
                                Platform::Linux => {
                                    // Linux netstat输出解析
                                    if line.contains("tcp") {
                                        tcp_count += 1;
                                        if line.contains("LISTEN") {
                                            listening_count += 1;
                                        } else if line.contains("ESTABLISHED") {
                                            established_count += 1;
                                        }
                                    } else if line.contains("udp") {
                                        udp_count += 1;
                                    }
                                },
                                Platform::Windows => {
                                    // Windows netstat输出解析
                                    if line.contains("TCP") {
                                        tcp_count += 1;
                                        if line.contains("LISTENING") {
                                            listening_count += 1;
                                        } else if line.contains("ESTABLISHED") {
                                            established_count += 1;
                                        }
                                    } else if line.contains("UDP") {
                                        udp_count += 1;
                                    }
                                },
                                Platform::Unknown => {
                                    // 未知平台，尝试通用解析
                                    if line.contains("tcp") || line.contains("TCP") {
                                        tcp_count += 1;
                                        if line.contains("LISTEN") || line.contains("LISTENING") {
                                            listening_count += 1;
                                        } else if line.contains("ESTABLISHED") {
                                            established_count += 1;
                                        }
                                    } else if line.contains("udp") || line.contains("UDP") {
                                        udp_count += 1;
                                    }
                                }
                            }
                        }

                        Ok(FunctionResult {
                            name: "sys-netstat".to_string(),
                            result: json!({
                                "netstat_output": lines,
                                "connection_stats": {
                                    "tcp_connections": tcp_count,
                                    "udp_connections": udp_count,
                                    "listening_ports": listening_count,
                                    "established_connections": established_count
                                },
                                "show_tcp": show_tcp,
                                "show_udp": show_udp,
                                "platform": platform_name_str,
                                "command": {
                                    "executable": command,
                                    "arguments": command_args
                                },
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
                        name: "sys-netstat".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to get network statistics: {}", e)),
                    }),
                }
            }

            "sys-diagnose" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let mode = args
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("standard");

                // 根据模式执行不同程度的诊断
                let timeout = match mode {
                    "quick" => 10,
                    "standard" => 20,
                    "deep" => 30,
                    _ => 20,
                };

                // 执行综合诊断命令组合
                let diagnostic_results =
                    self.perform_comprehensive_diagnosis(mode, timeout).await?;

                Ok(FunctionResult {
                    name: "sys-diagnose".to_string(),
                    result: json!({
                        "diagnosis_mode": mode,
                        "results": diagnostic_results,
                        "success": true
                    }),
                    error: None,
                })
            }

            _ => Err(OrionAiReason::from_logic("Unknown analysis function".to_string()).to_err()),
        }
    }

    fn supported_functions(&self) -> Vec<String> {
        vec![
            "sys-iostat".to_string(),
            "sys-netstat".to_string(),
            "sys-diagnose".to_string(),
        ]
    }

    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
        create_analysis_functions()
            .into_iter()
            .find(|f| f.name == function_name)
    }
}

impl AnalysisExecutor {
    async fn perform_comprehensive_diagnosis(
        &self,
        mode: &str,
        _timeout: u64,
    ) -> AiResult<serde_json::Value> {
        let mut results = serde_json::Map::new();

        // 检测当前平台
        let platform = detect_platform();
        let platform_name_str = platform_name(&platform);

        // 基础系统信息 - 使用平台特定的uptime命令
        let (uptime_command, uptime_args) = get_platform_specific_command(CommandType::Uptime, &platform);
        if let Ok(output) = execute_command_with_timeout(&uptime_command, &uptime_args, 5).await {
            results.insert(
                "uptime".to_string(),
                json!({
                    "data": String::from_utf8_lossy(&output.stdout).trim(),
                    "platform": platform_name_str,
                    "command": {
                        "executable": uptime_command,
                        "arguments": uptime_args
                    },
                    "success": output.status.success()
                }),
            );
        }

        // 内存信息 - 使用平台特定的内存命令
        let (mem_command, mem_args) = get_platform_specific_command(CommandType::MemInfo, &platform);
        if let Ok(output) = execute_command_with_timeout(&mem_command, &mem_args, 5).await {
            let mut mem_result = json!({
                "data": String::from_utf8_lossy(&output.stdout).trim(),
                "platform": platform_name_str,
                "command": {
                    "executable": mem_command,
                    "arguments": mem_args
                },
                "success": output.status.success()
            });

            // 如果是macOS，解析vm_stat输出
            if platform == Platform::MacOS {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let parsed_mem = parse_vmstat_output(&output_str);
                mem_result["parsed"] = json!(parsed_mem);
            }

            results.insert("memory".to_string(), mem_result);
        }

        // 进程信息 (仅在标准模式及以上)
        if mode != "quick" {
            let (ps_command, mut ps_args) = get_platform_specific_command(CommandType::ProcessList, &platform);
            
            // 根据平台添加排序参数
            match platform {
                Platform::MacOS => {
                    ps_args.push("-c");
                    ps_args.push("-o");
                    ps_args.push("%cpu,comm");
                },
                Platform::Linux => {
                    ps_args.push("--sort=-%cpu");
                },
                Platform::Windows => {
                    ps_args.push("/fo");
                    ps_args.push("csv");
                },
                Platform::Unknown => {
                    // 未知平台，尝试通用参数
                    ps_args.push("--sort=-%cpu");
                }
            }

            if let Ok(output) = execute_command_with_timeout(&ps_command, &ps_args, 10).await {
                let cpu_output = String::from_utf8_lossy(&output.stdout);
                let cpu_lines: Vec<String> = cpu_output.lines().take(11).map(|s| s.to_string()).collect();

                results.insert(
                    "top_cpu_processes".to_string(),
                    json!({
                        "data": cpu_lines,
                        "platform": platform_name_str,
                        "command": {
                            "executable": ps_command,
                            "arguments": ps_args
                        },
                        "success": output.status.success()
                    }),
                );
            }
        }

        // I/O统计 (仅在深度模式)
        if mode == "deep" {
            // I/O统计
            let (io_command, mut io_args) = get_platform_specific_command(CommandType::Iostat, &platform);
            
            // 根据平台添加参数
            match platform {
                Platform::MacOS | Platform::Linux => {
                    io_args.push("1");
                    io_args.push("2");
                },
                Platform::Windows => {
                    io_args.push("1");
                },
                Platform::Unknown => {
                    // 未知平台，尝试通用参数
                    io_args.push("1");
                    io_args.push("2");
                }
            }

            if let Ok(output) = execute_command_with_timeout(&io_command, &io_args, 10).await {
                results.insert(
                    "io_stats".to_string(),
                    json!({
                        "data": String::from_utf8_lossy(&output.stdout).trim(),
                        "platform": platform_name_str,
                        "command": {
                            "executable": io_command,
                            "arguments": io_args
                        },
                        "success": output.status.success()
                    }),
                );
            }

            // 网络统计
            let (net_command, net_args) = get_platform_specific_command(CommandType::NetStat, &platform);
            if let Ok(output) = execute_command_with_timeout(&net_command, &net_args, 10).await {
                results.insert(
                    "network_stats".to_string(),
                    json!({
                        "data": String::from_utf8_lossy(&output.stdout).trim(),
                        "platform": platform_name_str,
                        "command": {
                            "executable": net_command,
                            "arguments": net_args
                        },
                        "success": output.status.success()
                    }),
                );
            }
        }

        Ok(json!(results))
    }
}

pub fn create_analysis_functions() -> Vec<FunctionDefinition> {
    vec![
        FunctionDefinition {
            name: "sys-iostat".to_string(),
            description: "显示I/O统计信息，支持多采样".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "count".to_string(),
                    description: "采样次数，默认为2，最大为5".to_string(),
                    r#type: "number".to_string(),
                    required: false,
                },
                FunctionParameter {
                    name: "interval".to_string(),
                    description: "采样间隔（秒），默认为1，最大为3".to_string(),
                    r#type: "number".to_string(),
                    required: false,
                },
            ],
        },
        FunctionDefinition {
            name: "sys-netstat".to_string(),
            description: "显示网络连接统计信息".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "show_tcp".to_string(),
                    description: "显示TCP连接，默认为true".to_string(),
                    r#type: "boolean".to_string(),
                    required: false,
                },
                FunctionParameter {
                    name: "show_udp".to_string(),
                    description: "显示UDP连接，默认为false".to_string(),
                    r#type: "boolean".to_string(),
                    required: false,
                },
            ],
        },
        FunctionDefinition {
            name: "sys-diagnose".to_string(),
            description: "执行综合系统诊断，支持不同深度模式".to_string(),
            parameters: vec![FunctionParameter {
                name: "mode".to_string(),
                description:
                    "诊断模式: 'quick'(快速), 'standard'(标准), 'deep'(深度)，默认为'standard'"
                        .to_string(),
                r#type: "string".to_string(),
                required: false,
            }],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analysis_executor() {
        let executor = AnalysisExecutor;

        // 测试支持的函数列表
        let functions = executor.supported_functions();
        assert!(functions.contains(&"sys-iostat".to_string()));
        assert!(functions.contains(&"sys-netstat".to_string()));
        assert!(functions.contains(&"sys-diagnose".to_string()));
    }

    #[tokio::test]
    async fn test_create_analysis_functions() {
        let functions = create_analysis_functions();
        assert_eq!(functions.len(), 3);

        // 验证 iostat 函数定义
        let iostat_func = functions.iter().find(|f| f.name == "sys-iostat").unwrap();
        assert_eq!(iostat_func.description, "显示I/O统计信息，支持多采样");
        assert_eq!(iostat_func.parameters.len(), 2);
        assert_eq!(iostat_func.parameters[0].name, "count");
        assert!(!iostat_func.parameters[0].required);
        assert_eq!(iostat_func.parameters[1].name, "interval");
        assert!(!iostat_func.parameters[1].required);

        // 验证 netstat 函数定义
        let netstat_func = functions.iter().find(|f| f.name == "sys-netstat").unwrap();
        assert_eq!(netstat_func.description, "显示网络连接统计信息");
        assert_eq!(netstat_func.parameters.len(), 2);
        assert_eq!(netstat_func.parameters[0].name, "show_tcp");
        assert!(!netstat_func.parameters[0].required);
        assert_eq!(netstat_func.parameters[1].name, "show_udp");
        assert!(!netstat_func.parameters[1].required);

        // 验证 diagnose 函数定义
        let diagnose_func = functions.iter().find(|f| f.name == "sys-diagnose").unwrap();
        assert_eq!(
            diagnose_func.description,
            "执行综合系统诊断，支持不同深度模式"
        );
        assert_eq!(diagnose_func.parameters.len(), 1);
        assert_eq!(diagnose_func.parameters[0].name, "mode");
        assert!(!diagnose_func.parameters[0].required);
    }
}
