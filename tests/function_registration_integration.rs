use orion_ai::*;
use std::sync::Arc;

#[tokio::test]
async fn test_dynamic_function_registration() {
    // 重置注册表以确保干净的测试环境
    GlobalFunctionRegistry::reset();
    assert!(GlobalFunctionRegistry::initialize().is_ok());

    // 创建并注册自定义函数
    let custom_function = FunctionDefinition {
        name: "test-custom-function".to_string(),
        description: "Test custom function".to_string(),
        parameters: vec![],
    };

    // 测试注册函数
    assert!(GlobalFunctionRegistry::register_function(custom_function.clone()).is_ok());

    // 验证注册成功
    let registry = GlobalFunctionRegistry::get_registry().unwrap();
    assert!(registry.contains_function("test-custom-function"));

    // 测试重复注册应该失败
    assert!(GlobalFunctionRegistry::register_function(custom_function).is_err());
}

#[tokio::test]
async fn test_mixed_builtin_and_custom_tools() {
    // 重置注册表以确保干净的测试环境
    GlobalFunctionRegistry::reset();
    assert!(GlobalFunctionRegistry::initialize().is_ok());

    // 注册自定义工具
    let custom_function = FunctionDefinition {
        name: "mixed-custom-tool".to_string(),
        description: "Custom tool for mixed test".to_string(),
        parameters: vec![],
    };

    struct MixedToolExecutor;

    #[async_trait::async_trait]
    impl FunctionExecutor for MixedToolExecutor {
        async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
            Ok(FunctionResult {
                name: function_call.function.name.clone(),
                result: serde_json::json!({"mixed": "custom_result"}),
                error: None,
            })
        }

        fn supported_functions(&self) -> Vec<String> {
            vec!["mixed-custom-tool".to_string()]
        }

        fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
            if function_name == "mixed-custom-tool" {
                Some(FunctionDefinition {
                    name: "mixed-custom-tool".to_string(),
                    description: "Custom tool for mixed test".to_string(),
                    parameters: vec![],
                })
            } else {
                None
            }
        }
    }

    let executor = Arc::new(MixedToolExecutor);

    // 注册自定义工具
    assert!(GlobalFunctionRegistry::register_function(custom_function).is_ok());
    assert!(
        GlobalFunctionRegistry::register_executor("mixed-custom-tool".to_string(), executor)
            .is_ok()
    );

    // 创建包含内置工具和自定义工具的执行单元
    let ai = AiExecUnitBuilder::new(std::collections::HashMap::new().into())
        .with_tools(vec![
            "git-status".to_string(),
            "mixed-custom-tool".to_string(),
            "fs-ls".to_string(),
        ])
        .build();

    // 验证两种工具都能正常工作
    let ai_unit = ai.unwrap();
    let registry = ai_unit.registry();
    let function_names = registry.get_supported_function_names();

    assert!(function_names.contains(&"git-status".to_string()));
    assert!(function_names.contains(&"mixed-custom-tool".to_string()));
    assert!(function_names.contains(&"fs-ls".to_string()));
}

#[tokio::test]
async fn test_concurrent_registration() {
    // 重置注册表以确保干净的测试环境
    GlobalFunctionRegistry::reset();
    assert!(GlobalFunctionRegistry::initialize().is_ok());

    use tokio::task::JoinSet;

    let mut join_set = JoinSet::new();

    // 并发注册多个工具
    for i in 0..10 {
        join_set.spawn(async move {
            let function_name = format!("concurrent-tool-{}", i);

            let function = FunctionDefinition {
                name: function_name.clone(),
                description: format!("Concurrent test tool {}", i),
                parameters: vec![],
            };

            struct ConcurrentExecutor {
                function_name: String,
            }

            #[async_trait::async_trait]
            impl FunctionExecutor for ConcurrentExecutor {
                async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
                    Ok(FunctionResult {
                        name: function_call.function.name.clone(),
                        result: serde_json::json!({"concurrent": "ok"}),
                        error: None,
                    })
                }

                fn supported_functions(&self) -> Vec<String> {
                    vec![self.function_name.clone()]
                }

                fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
                    if function_name == self.function_name {
                        Some(FunctionDefinition {
                            name: self.function_name.clone(),
                            description: format!("Concurrent test tool for {}", function_name),
                            parameters: vec![],
                        })
                    } else {
                        None
                    }
                }
            }

            let executor = Arc::new(ConcurrentExecutor {
                function_name: function_name.clone(),
            });

            // 并发注册
            let reg_result = GlobalFunctionRegistry::register_function(function);
            let exec_result = GlobalFunctionRegistry::register_executor(function_name, executor);

            (reg_result.is_ok(), exec_result.is_ok())
        });
    }

    // 等待所有并发操作完成
    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok((reg_ok, exec_ok)) => {
                results.push((reg_ok, exec_ok));
            }
            Err(e) => {
                panic!("Task failed: {}", e);
            }
        }
    }

    // 验证所有并发注册都成功
    for (reg_ok, exec_ok) in results {
        assert!(reg_ok, "Function registration failed");
        assert!(exec_ok, "Executor registration failed");
    }

    // 验证所有函数都已注册
    let registry = GlobalFunctionRegistry::get_registry().unwrap();
    let function_names = registry.get_function_names();

    for i in 0..10 {
        let expected_name = format!("concurrent-tool-{}", i);
        assert!(
            function_names.contains(&expected_name),
            "Function {} not found",
            expected_name
        );
    }
}

#[tokio::test]
async fn test_tool_set_integration() {
    // 重置注册表以确保干净的测试环境
    GlobalFunctionRegistry::reset();
    assert!(GlobalFunctionRegistry::initialize().is_ok());

    struct IntegrationToolSetExecutor;

    #[async_trait::async_trait]
    impl FunctionExecutor for IntegrationToolSetExecutor {
        async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
            Ok(FunctionResult {
                name: function_call.function.name.clone(),
                result: serde_json::json!({"integration": "tool_set_result"}),
                error: None,
            })
        }

        fn supported_functions(&self) -> Vec<String> {
            vec![
                "integration-tool-1".to_string(),
                "integration-tool-2".to_string(),
                "integration-tool-3".to_string(),
            ]
        }

        fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
            match function_name {
                "integration-tool-1" => Some(FunctionDefinition {
                    name: "integration-tool-1".to_string(),
                    description: "Integration tool 1".to_string(),
                    parameters: vec![],
                }),
                "integration-tool-2" => Some(FunctionDefinition {
                    name: "integration-tool-2".to_string(),
                    description: "Integration tool 2".to_string(),
                    parameters: vec![],
                }),
                "integration-tool-3" => Some(FunctionDefinition {
                    name: "integration-tool-3".to_string(),
                    description: "Integration tool 3".to_string(),
                    parameters: vec![],
                }),
                _ => None,
            }
        }
    }

    let functions = vec![
        FunctionDefinition {
            name: "integration-tool-1".to_string(),
            description: "Integration tool 1".to_string(),
            parameters: vec![],
        },
        FunctionDefinition {
            name: "integration-tool-2".to_string(),
            description: "Integration tool 2".to_string(),
            parameters: vec![],
        },
        FunctionDefinition {
            name: "integration-tool-3".to_string(),
            description: "Integration tool 3".to_string(),
            parameters: vec![],
        },
    ];

    let executor = Arc::new(IntegrationToolSetExecutor);

    // 批量注册工具集
    assert!(GlobalFunctionRegistry::register_tool_set(functions, executor).is_ok());

    // 验证所有工具都已注册并可以正常工作
    let registry = GlobalFunctionRegistry::get_registry().unwrap();

    for tool_name in &[
        "integration-tool-1",
        "integration-tool-2",
        "integration-tool-3",
    ] {
        assert!(
            registry.contains_function(tool_name),
            "Tool {} not found",
            tool_name
        );

        // 测试工具调用
        let function_call = FunctionCall {
            index: Some(0),
            id: format!("call_{}", tool_name),
            r#type: "function".to_string(),
            function: crate::provider::FunctionCallInfo {
                name: tool_name.to_string(),
                arguments: "{}".to_string(),
            },
        };

        let result = registry.execute_function(&function_call).await.unwrap();
        assert!(
            result.error.is_none(),
            "Tool {} execution failed",
            tool_name
        );
        assert_eq!(result.result["integration"], "tool_set_result");
    }
}
