use orion_error::{ToStructError, UvsLogicFrom};
use serde_json::json;

use crate::{
    AiResult, FunctionCall, FunctionDefinition, FunctionExecutor, FunctionParameter,
    FunctionResult, error::OrionAiReason,
};

// 解析函数参数的辅助函数
fn parse_function_arguments(
    arguments: &str,
) -> AiResult<serde_json::Map<String, serde_json::Value>> {
    if arguments.trim().is_empty() || arguments == "{}" {
        return Ok(serde_json::Map::new());
    }

    let parsed: serde_json::Value = serde_json::from_str(arguments).map_err(|e| {
        OrionAiReason::from_logic(format!("Failed to parse arguments: {}", e)).to_err()
    })?;

    match parsed {
        serde_json::Value::Object(map) => Ok(map),
        _ => Err(OrionAiReason::from_logic("Arguments must be an object".to_string()).to_err()),
    }
}

// Git 函数执行器
pub struct GitFunctionExecutor;

#[async_trait::async_trait]
impl FunctionExecutor for GitFunctionExecutor {
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        match function_call.function.name.as_str() {
            "git-status" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");

                match tokio::process::Command::new("git")
                    .args(["status", "--porcelain"])
                    .current_dir(path)
                    .output()
                    .await
                {
                    Ok(output) => {
                        let status = String::from_utf8_lossy(&output.stdout).to_string();
                        Ok(FunctionResult {
                            name: "git-status".to_string(),
                            result: json!({
                                "status": status,
                                "has_changes": !status.trim().is_empty()
                            }),
                            error: None,
                        })
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "git-status".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to get git status: {}", e)),
                    }),
                }
            }

            "git-diff" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                let staged = args
                    .get("staged")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let mut args = vec!["diff"];
                if staged {
                    args.push("--staged");
                }
                args.push(path);

                match tokio::process::Command::new("git")
                    .args(args)
                    .current_dir(".")
                    .output()
                    .await
                {
                    Ok(output) => {
                        let diff = String::from_utf8_lossy(&output.stdout).to_string();
                        Ok(FunctionResult {
                            name: "git-diff".to_string(),
                            result: json!({
                                "diff": diff,
                                "has_changes": !diff.trim().is_empty()
                            }),
                            error: None,
                        })
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "git-diff".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to get git diff: {}", e)),
                    }),
                }
            }

            "git-add" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let files = args
                    .get("files")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        OrionAiReason::from_logic("TODO: files parameter required".to_string())
                            .to_err()
                    })?;

                let file_list: Vec<String> = files
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect();

                match tokio::process::Command::new("git")
                    .args(["add"])
                    .args(file_list)
                    .output()
                    .await
                {
                    Ok(output) => {
                        if output.status.success() {
                            Ok(FunctionResult {
                                name: "git-add".to_string(),
                                result: json!({
                                    "success": true,
                                    "message": "Files added successfully"
                                }),
                                error: None,
                            })
                        } else {
                            let error_msg = String::from_utf8_lossy(&output.stderr);
                            Ok(FunctionResult {
                                name: "git-add".to_string(),
                                result: serde_json::Value::Null,
                                error: Some(error_msg.to_string()),
                            })
                        }
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "git-add".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to add files: {}", e)),
                    }),
                }
            }

            "git-commit" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let message = args
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        OrionAiReason::from_logic("TODO: message parameter required".to_string())
                            .to_err()
                    })?;

                match tokio::process::Command::new("git")
                    .args(["commit", "-m", message])
                    .output()
                    .await
                {
                    Ok(output) => {
                        if output.status.success() {
                            Ok(FunctionResult {
                                name: "git-commit".to_string(),
                                result: json!({
                                    "success": true,
                                    "message": "Commit created successfully"
                                }),
                                error: None,
                            })
                        } else {
                            let error_msg = String::from_utf8_lossy(&output.stderr);
                            Ok(FunctionResult {
                                name: "git-commit".to_string(),
                                result: serde_json::Value::Null,
                                error: Some(error_msg.to_string()),
                            })
                        }
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "git-commit".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to create commit: {}", e)),
                    }),
                }
            }

            "git-push" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let remote = args
                    .get("remote")
                    .and_then(|v| v.as_str())
                    .unwrap_or("origin");
                let branch = args
                    .get("branch")
                    .and_then(|v| v.as_str())
                    .unwrap_or("HEAD");

                match tokio::process::Command::new("git")
                    .args(["push", remote, branch])
                    .output()
                    .await
                {
                    Ok(output) => {
                        if output.status.success() {
                            Ok(FunctionResult {
                                name: "git-push".to_string(),
                                result: json!({
                                    "success": true,
                                    "message": format!("Pushed to {}/{}", remote, branch)
                                }),
                                error: None,
                            })
                        } else {
                            let error_msg = String::from_utf8_lossy(&output.stderr);
                            Ok(FunctionResult {
                                name: "git-push".to_string(),
                                result: serde_json::Value::Null,
                                error: Some(error_msg.to_string()),
                            })
                        }
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "git-push".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to push: {}", e)),
                    }),
                }
            }

            _ => Err(OrionAiReason::from_logic("TODO: unknown function".to_string()).to_err()),
        }
    }

    fn supported_functions(&self) -> Vec<String> {
        vec![
            "git-status".to_string(),
            "git-diff".to_string(),
            "git-add".to_string(),
            "git-commit".to_string(),
            "git-push".to_string(),
        ]
    }

    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
        create_git_functions()
            .into_iter()
            .find(|f| f.name == function_name)
    }
}

pub fn create_git_functions() -> Vec<FunctionDefinition> {
    vec![
        FunctionDefinition {
            name: "git-status".to_string(),
            description: "获取Git仓库状态".to_string(),
            parameters: vec![FunctionParameter {
                name: "path".to_string(),
                description: "仓库路径，默认为当前目录".to_string(),
                r#type: "string".to_string(),
                required: false,
            }],
        },
        FunctionDefinition {
            name: "git-diff".to_string(),
            description: "显示Git仓库的变更差异".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "path".to_string(),
                    description: "Git仓库路径，默认为当前目录".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                },
                FunctionParameter {
                    name: "staged".to_string(),
                    description: "是否只显示暂存的变更，默认为false".to_string(),
                    r#type: "boolean".to_string(),
                    required: false,
                },
            ],
        },
        FunctionDefinition {
            name: "git-add".to_string(),
            description: "添加文件到Git暂存区".to_string(),
            parameters: vec![FunctionParameter {
                name: "files".to_string(),
                description: "要添加的文件列表，支持通配符".to_string(),
                r#type: "array".to_string(),
                required: true,
            }],
        },
        FunctionDefinition {
            name: "git-commit".to_string(),
            description: "创建Git提交".to_string(),
            parameters: vec![FunctionParameter {
                name: "message".to_string(),
                description: "提交消息".to_string(),
                r#type: "string".to_string(),
                required: true,
            }],
        },
        FunctionDefinition {
            name: "git-push".to_string(),
            description: "推送提交到远程仓库".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "remote".to_string(),
                    description: "远程仓库名称，默认为origin".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                },
                FunctionParameter {
                    name: "branch".to_string(),
                    description: "分支名称，默认为当前分支".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                },
            ],
        },
    ]
}
