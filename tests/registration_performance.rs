use orion_ai::*;
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
async fn test_registration_performance() {
    GlobalFunctionRegistry::reset();
    assert!(GlobalFunctionRegistry::initialize().is_ok());

    // 测试批量注册大量工具的性能
    let start = Instant::now();

    struct PerformanceTestExecutor;

    #[async_trait::async_trait]
    impl FunctionExecutor for PerformanceTestExecutor {
        async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
            Ok(FunctionResult {
                name: function_call.function.name.clone(),
                result: serde_json::json!({"performance": "test"}),
                error: None,
            })
        }

        fn supported_functions(&self) -> Vec<String> {
            (0..100).map(|i| format!("perf-function-{}", i)).collect()
        }

        fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
            function_name
                .starts_with("perf-function-")
                .then(|| FunctionDefinition {
                    name: function_name.to_string(),
                    description: "Performance test function".to_string(),
                    parameters: vec![],
                })
        }
    }

    let executor = Arc::new(PerformanceTestExecutor);
    let functions: Vec<FunctionDefinition> = (0..100)
        .map(|i| FunctionDefinition {
            name: format!("perf-function-{}", i),
            description: "Performance test function".to_string(),
            parameters: vec![],
        })
        .collect();

    // 批量注册100个工具
    assert!(GlobalFunctionRegistry::register_tool_set(functions, executor).is_ok());

    let registration_time = start.elapsed();
    println!("批量注册100个工具耗时: {:?}", registration_time);

    // 验证所有工具都已注册
    let registry = GlobalFunctionRegistry::get_registry().unwrap();
    let function_names = registry.get_function_names();
    assert!(function_names.len() >= 100);

    // 验证性能在合理范围内（应该小于100ms）
    assert!(registration_time.as_millis() < 100);
}

#[tokio::test]
async fn test_individual_registration_performance() {
    GlobalFunctionRegistry::reset();
    assert!(GlobalFunctionRegistry::initialize().is_ok());

    struct IndividualPerfExecutor;

    #[async_trait::async_trait]
    impl FunctionExecutor for IndividualPerfExecutor {
        async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
            Ok(FunctionResult {
                name: function_call.function.name.clone(),
                result: serde_json::json!({"individual": "perf"}),
                error: None,
            })
        }

        fn supported_functions(&self) -> Vec<String> {
            vec!["individual-perf-function".to_string()]
        }

        fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
            if function_name == "individual-perf-function" {
                Some(FunctionDefinition {
                    name: "individual-perf-function".to_string(),
                    description: "Individual performance test function".to_string(),
                    parameters: vec![],
                })
            } else {
                None
            }
        }
    }

    let executor = Arc::new(IndividualPerfExecutor);
    let function = FunctionDefinition {
        name: "individual-perf-function".to_string(),
        description: "Individual performance test function".to_string(),
        parameters: vec![],
    };

    // 测试单个注册的性能
    let start = Instant::now();

    // 分别注册
    assert!(GlobalFunctionRegistry::register_function(function).is_ok());
    assert!(
        GlobalFunctionRegistry::register_executor("individual-perf-function".to_string(), executor)
            .is_ok()
    );

    let individual_time = start.elapsed();
    println!("单个工具注册耗时: {:?}", individual_time);

    // 验证注册成功
    let registry = GlobalFunctionRegistry::get_registry().unwrap();
    assert!(registry.contains_function("individual-perf-function"));

    // 验证性能在合理范围内（应该小于10ms）
    assert!(individual_time.as_millis() < 10);
}

#[tokio::test]
async fn test_concurrent_registration_performance() {
    GlobalFunctionRegistry::reset();
    assert!(GlobalFunctionRegistry::initialize().is_ok());

    use tokio::task::JoinSet;

    let mut join_set = JoinSet::new();
    let start = Instant::now();

    // 并发注册50个不同的工具集
    for set_id in 0..50 {
        join_set.spawn(async move {
            struct ConcurrentPerfExecutor {
                set_id: usize,
            }

            #[async_trait::async_trait]
            impl FunctionExecutor for ConcurrentPerfExecutor {
                async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
                    Ok(FunctionResult {
                        name: function_call.function.name.clone(),
                        result: serde_json::json!({"concurrent": self.set_id}),
                        error: None,
                    })
                }

                fn supported_functions(&self) -> Vec<String> {
                    (0..5)
                        .map(|i| format!("concurrent-perf-{}-{}", self.set_id, i))
                        .collect()
                }

                fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
                    function_name
                        .starts_with(&format!("concurrent-perf-{}-", self.set_id))
                        .then(|| FunctionDefinition {
                            name: function_name.to_string(),
                            description: "Concurrent performance test function".to_string(),
                            parameters: vec![],
                        })
                }
            }

            let executor = Arc::new(ConcurrentPerfExecutor { set_id });
            let functions: Vec<FunctionDefinition> = (0..5)
                .map(|i| FunctionDefinition {
                    name: format!("concurrent-perf-{}-{}", set_id, i),
                    description: "Concurrent performance test function".to_string(),
                    parameters: vec![],
                })
                .collect();

            // 批量注册工具集
            GlobalFunctionRegistry::register_tool_set(functions, executor)
        });
    }

    // 等待所有并发注册完成
    let mut success_count = 0;
    let mut failure_count = 0;

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(reg_result) => {
                if reg_result.is_ok() {
                    success_count += 1;
                } else {
                    failure_count += 1;
                    println!("并发注册失败: {:?}", reg_result.err());
                }
            }
            Err(e) => {
                failure_count += 1;
                println!("并发任务失败: {}", e);
            }
        }
    }

    let concurrent_time = start.elapsed();
    println!("并发注册50个工具集耗时: {:?}", concurrent_time);
    println!("成功: {}, 失败: {}", success_count, failure_count);

    // 验证所有并发注册都成功
    assert_eq!(failure_count, 0, "有并发注册失败");
    assert_eq!(success_count, 50, "并发注册数量不正确");

    // 验证所有函数都已注册
    let registry = GlobalFunctionRegistry::get_registry().unwrap();
    let function_names = registry.get_function_names();

    let mut expected_count = 0;
    for set_id in 0..50 {
        for i in 0..5 {
            let expected_name = format!("concurrent-perf-{}-{}", set_id, i);
            assert!(
                function_names.contains(&expected_name),
                "函数 {} 未找到",
                expected_name
            );
            expected_count += 1;
        }
    }

    assert_eq!(function_names.len(), expected_count);

    // 验证性能在合理范围内（应该小于200ms）
    assert!(concurrent_time.as_millis() < 200);
}

#[tokio::test]
async fn test_unregistration_performance() {
    GlobalFunctionRegistry::reset();
    assert!(GlobalFunctionRegistry::initialize().is_ok());

    // 先注册大量工具
    struct UnregPerfExecutor;

    #[async_trait::async_trait]
    impl FunctionExecutor for UnregPerfExecutor {
        async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
            Ok(FunctionResult {
                name: function_call.function.name.clone(),
                result: serde_json::json!({"unreg": "test"}),
                error: None,
            })
        }

        fn supported_functions(&self) -> Vec<String> {
            (0..50)
                .map(|i| format!("unreg-perf-function-{}", i))
                .collect()
        }

        fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
            function_name
                .starts_with("unreg-perf-function-")
                .then(|| FunctionDefinition {
                    name: function_name.to_string(),
                    description: "Unregistration performance test function".to_string(),
                    parameters: vec![],
                })
        }
    }

    let executor = Arc::new(UnregPerfExecutor);
    let functions: Vec<FunctionDefinition> = (0..50)
        .map(|i| FunctionDefinition {
            name: format!("unreg-perf-function-{}", i),
            description: "Unregistration performance test function".to_string(),
            parameters: vec![],
        })
        .collect();

    // 注册50个工具
    assert!(GlobalFunctionRegistry::register_tool_set(functions, executor).is_ok());

    // 测试批量注销的性能
    let start = Instant::now();

    for i in 0..50 {
        let function_name = format!("unreg-perf-function-{}", i);
        assert!(GlobalFunctionRegistry::unregister_function(&function_name).is_ok());
    }

    let unregistration_time = start.elapsed();
    println!("批量注销50个工具耗时: {:?}", unregistration_time);

    // 验证所有工具都已注销
    let registry = GlobalFunctionRegistry::get_registry().unwrap();
    let function_names = registry.get_function_names();

    for i in 0..50 {
        let function_name = format!("unreg-perf-function-{}", i);
        assert!(
            !function_names.contains(&function_name),
            "函数 {} 仍然存在",
            function_name
        );
    }

    // 验证性能在合理范围内（应该小于50ms）
    assert!(unregistration_time.as_millis() < 50);
}

#[tokio::test]
async fn test_registry_query_performance() {
    GlobalFunctionRegistry::reset();
    assert!(GlobalFunctionRegistry::initialize().is_ok());

    // 先注册大量工具用于查询性能测试
    struct QueryPerfExecutor;

    #[async_trait::async_trait]
    impl FunctionExecutor for QueryPerfExecutor {
        async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
            Ok(FunctionResult {
                name: function_call.function.name.clone(),
                result: serde_json::json!({"query": "perf"}),
                error: None,
            })
        }

        fn supported_functions(&self) -> Vec<String> {
            (0..200)
                .map(|i| format!("query-perf-function-{}", i))
                .collect()
        }

        fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
            function_name
                .starts_with("query-perf-function-")
                .then(|| FunctionDefinition {
                    name: function_name.to_string(),
                    description: "Query performance test function".to_string(),
                    parameters: vec![],
                })
        }
    }

    let executor = Arc::new(QueryPerfExecutor);
    let functions: Vec<FunctionDefinition> = (0..200)
        .map(|i| FunctionDefinition {
            name: format!("query-perf-function-{}", i),
            description: "Query performance test function".to_string(),
            parameters: vec![],
        })
        .collect();

    // 注册200个工具
    assert!(GlobalFunctionRegistry::register_tool_set(functions, executor).is_ok());

    let registry = GlobalFunctionRegistry::get_registry().unwrap();

    // 测试查询性能
    let start = Instant::now();

    // 执行1000次查询
    for _ in 0..1000 {
        // 随机查询一些函数
        for i in [0, 50, 100, 150, 199] {
            let function_name = format!("query-perf-function-{}", i);
            let _contains = registry.contains_function(&function_name);
            let _schema = registry.get_function(&function_name);
            let _executor = registry.get_executor(&function_name);
        }
    }

    let query_time = start.elapsed();
    println!("执行5000次查询操作耗时: {:?}", query_time);

    // 验证性能在合理范围内（应该小于100ms）
    assert!(query_time.as_millis() < 100);
}

#[tokio::test]
async fn test_memory_usage_profile() {
    GlobalFunctionRegistry::reset();
    assert!(GlobalFunctionRegistry::initialize().is_ok());

    // 测试内存使用情况
    let initial_registry = GlobalFunctionRegistry::get_registry().unwrap();
    let initial_function_count = initial_registry.get_function_names().len();
    println!("初始函数数量: {}", initial_function_count);

    struct MemoryPerfExecutor;

    #[async_trait::async_trait]
    impl FunctionExecutor for MemoryPerfExecutor {
        async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
            Ok(FunctionResult {
                name: function_call.function.name.clone(),
                result: serde_json::json!({"memory": "test"}),
                error: None,
            })
        }

        fn supported_functions(&self) -> Vec<String> {
            (0..1000)
                .map(|i| format!("memory-perf-function-{}", i))
                .collect()
        }

        fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
            function_name
                .starts_with("memory-perf-function-")
                .then(|| FunctionDefinition {
                    name: function_name.to_string(),
                    description: "Memory performance test function".to_string(),
                    parameters: vec![],
                })
        }
    }

    let executor = Arc::new(MemoryPerfExecutor);
    let functions: Vec<FunctionDefinition> = (0..1000)
        .map(|i| FunctionDefinition {
            name: format!("memory-perf-function-{}", i),
            description: "Memory performance test function".to_string(),
            parameters: vec![],
        })
        .collect();

    // 注册1000个工具
    assert!(GlobalFunctionRegistry::register_tool_set(functions, executor).is_ok());

    let final_registry = GlobalFunctionRegistry::get_registry().unwrap();
    let final_function_count = final_registry.get_function_names().len();
    println!("最终函数数量: {}", final_function_count);

    // 验证函数数量正确
    assert_eq!(final_function_count, initial_function_count + 1000);

    // 验证所有新函数都存在
    for i in 0..1000 {
        let function_name = format!("memory-perf-function-{}", i);
        assert!(
            final_registry.contains_function(&function_name),
            "函数 {} 未找到",
            function_name
        );
    }

    // 清理：注销所有测试函数
    for i in 0..1000 {
        let function_name = format!("memory-perf-function-{}", i);
        let _ = GlobalFunctionRegistry::unregister_function(&function_name);
    }

    let cleanup_registry = GlobalFunctionRegistry::get_registry().unwrap();
    let cleanup_function_count = cleanup_registry.get_function_names().len();
    println!("清理后函数数量: {}", cleanup_function_count);

    // 验证清理效果
    assert_eq!(cleanup_function_count, initial_function_count);
}

#[tokio::test]
async fn test_auto_initialization_performance() {
    // 重置注册表
    GlobalFunctionRegistry::reset();

    // 测试自动初始化的性能
    let start = Instant::now();

    // 触发自动初始化（通过注册一个函数）
    let test_function = FunctionDefinition {
        name: "auto-init-test-function".to_string(),
        description: "Auto initialization test function".to_string(),
        parameters: vec![],
    };

    assert!(GlobalFunctionRegistry::register_function(test_function).is_ok());

    let auto_init_time = start.elapsed();
    println!("自动初始化并注册耗时: {:?}", auto_init_time);

    // 验证自动初始化成功
    let registry = GlobalFunctionRegistry::get_registry().unwrap();
    assert!(registry.contains_function("auto-init-test-function"));

    // 验证性能在合理范围内（应该小于50ms）
    assert!(auto_init_time.as_millis() < 50);
}
