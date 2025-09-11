use crate::{
    AiClient, AiResult, AiRoleID, FunctionResult, client::AiClientTrait,
    func::registry::FunctionRegistry, types::result::ExecutionResult,
    types::diagnosis::{DiagnosticConfig, DiagnosticDepth, DiagnosticReport, ProgressiveDiagnosis},
};
use getset::{Getters, MutGetters, Setters, WithSetters};

#[derive(Getters, MutGetters, Setters, WithSetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct AiExecUnit {
    client: AiClient,
    role: AiRoleID,
    registry: FunctionRegistry,
    diagnostic_config: Option<DiagnosticConfig>,
}

impl std::fmt::Debug for AiExecUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiExecUnit")
            .field("role", &self.role)
            .field(
                "registry",
                &format!("FunctionRegistry({})", self.registry.get_functions().len()),
            )
            .field("client", &"AiClient".to_string())
            .finish()
    }
}

impl AiExecUnit {
    /// 创建新的执行单元
    ///
    /// # 参数
    ///
    /// * `client` - AI客户端实例
    /// * `role` - AI角色标识
    /// * `registry` - 函数注册表
    pub fn new(client: AiClient, role: AiRoleID, registry: FunctionRegistry) -> Self {
        Self {
            client,
            role,
            registry,
            diagnostic_config: None,
        }
    }

    /// 创建新的执行单元（带诊断配置）
    ///
    /// # 参数
    ///
    /// * `client` - AI客户端实例
    /// * `role` - AI角色标识
    /// * `registry` - 函数注册表
    /// * `diagnostic_config` - 诊断配置
    pub fn with_diagnostic_config(
        client: AiClient,
        role: AiRoleID,
        registry: FunctionRegistry,
        diagnostic_config: DiagnosticConfig,
    ) -> Self {
        Self {
            client,
            role,
            registry,
            diagnostic_config: Some(diagnostic_config),
        }
    }

    pub async fn execute(&self, prompt: &str) -> AiResult<ExecutionResult> {
        let response = self.client.smart_role_request(&self.role, prompt).await?;
        Ok(ExecutionResult::new(response.content))
    }
    pub async fn execute_with_func(&self, prompt: &str) -> AiResult<ExecutionResult> {
        let response = self
            .client
            .role_funs_request(&self.role, prompt, self.registry().clone_functions())
            .await?;

        // 立即执行模式 (方案1): 检测到工具调用后立即执行
        // 注意：这是一个同步执行实现，AI会基于工具结果结束第一轮对话
        // TODO: 未来可扩展为支持多轮工具调用的链式执行模式
        let tool_results = if let Some(tool_calls) = &response.tool_calls {
            let mut results = Vec::new();

            for tool_call in tool_calls {
                // 使用函数注册表实际执行工具调用
                let execution_result = self.registry.execute_function(tool_call).await;

                match execution_result {
                    Ok(result) => {
                        results.push(FunctionResult {
                            name: tool_call.function.name.clone(),
                            result: result.result, // 实际执行结果
                            error: None,
                        });
                    }
                    Err(e) => {
                        results.push(FunctionResult {
                            name: tool_call.function.name.clone(),
                            result: serde_json::Value::Null,
                            error: Some(e.to_string()), // 记录错误信息
                        });
                    }
                }
            }
            results
        } else {
            Vec::new()
        };

        Ok(ExecutionResult::new(response.content).with_tool_calls(tool_results))
    }

    /// 执行系统诊断
    ///
    /// # 参数
    ///
    /// * `depth` - 诊断深度级别
    ///
    /// # 返回
    ///
    /// 返回诊断报告
    pub async fn execute_diagnosis(&self, depth: DiagnosticDepth) -> AiResult<DiagnosticReport> {
        let config = depth.to_config();
        self.execute_diagnosis_with_config(config).await
    }

    /// 执行系统诊断（使用自定义配置）
    ///
    /// # 参数
    ///
    /// * `config` - 诊断配置
    ///
    /// # 返回
    ///
    /// 返回诊断报告
    pub async fn execute_diagnosis_with_config(&self, config: DiagnosticConfig) -> AiResult<DiagnosticReport> {
        let diagnosis = ProgressiveDiagnosis::new(config);
        diagnosis
            .execute_progressive_diagnosis(&self.registry)
            .await
            .map_err(|e| crate::OrionAiReason::from_diagnosis(format!("诊断执行失败: {}", e)))
    }

    /// 执行快速健康检查
    ///
    /// # 返回
    ///
    /// 返回诊断报告
    pub async fn quick_health_check(&self) -> AiResult<DiagnosticReport> {
        self.execute_diagnosis(DiagnosticDepth::Basic).await
    }

    /// 执行标准诊断
    ///
    /// # 返回
    ///
    /// 返回诊断报告
    pub async fn standard_diagnosis(&self) -> AiResult<DiagnosticReport> {
        self.execute_diagnosis(DiagnosticDepth::Standard).await
    }

    /// 执行深度分析
    ///
    /// # 返回
    ///
    /// 返回诊断报告
    pub async fn deep_analysis(&self) -> AiResult<DiagnosticReport> {
        self.execute_diagnosis(DiagnosticDepth::Advanced).await
    }

    /// 消费执行单元，返回其组件
    ///
    /// # 返回
    ///
    /// 返回包含客户端、角色和函数注册表的元组
    pub fn into_components(self) -> (AiClient, AiRoleID, FunctionRegistry) {
        (self.client, self.role, self.registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{client::AiClientBuilder, config::AiConfig, types::diagnosis::{DiagnosticConfig, DiagnosticDepth}};

    #[tokio::test]
    async fn test_exec_unit_creation() {
        // 这个测试需要有效的AI配置，在实际环境中可能无法运行
        // 主要是测试编译和基本功能

        // 创建一个模拟的执行单元（在实际测试中需要有效的配置）
        let config = AiConfig::example();
        let client = AiClientBuilder::new(config).build().unwrap();
        let role = client.roles().default_role().clone();
        let registry = FunctionRegistry::new();

        let exec_unit = AiExecUnit::new(client, role.clone(), registry);

        assert_eq!(exec_unit.role(), &role);
        assert!(exec_unit.diagnostic_config().is_none());
    }

    #[tokio::test]
    async fn test_exec_unit_creation_with_diagnostic_config() {
        // 测试带诊断配置的执行单元创建
        let config = AiConfig::example();
        let client = AiClientBuilder::new(config).build().unwrap();
        let role = client.roles().default_role().clone();
        let registry = FunctionRegistry::new();
        let diagnostic_config = DiagnosticConfig::basic();

        let exec_unit = AiExecUnit::with_diagnostic_config(
            client, role.clone(), registry, diagnostic_config.clone(),
        );

        assert_eq!(exec_unit.role(), &role);
        assert_eq!(exec_unit.diagnostic_config(), &Some(diagnostic_config));
    }

    #[test]
    fn test_into_components() {
        // 测试into_components方法
        let config = AiConfig::example();
        let client = AiClientBuilder::new(config).build().unwrap();
        let role = client.roles().default_role().clone();
        let registry = FunctionRegistry::new();

        let exec_unit = AiExecUnit::new(client, role.clone(), registry);
        let (_returned_client, returned_role, _returned_registry) = exec_unit.into_components();

        // 验证返回的组件与原始组件相同
        assert_eq!(returned_role, role);
        // client 和 registry 的比较需要特殊的比较逻辑
    }

    #[test]
    fn test_with_registry() {
        // 测试with_registry方法
        let config = AiConfig::example();
        let client = AiClientBuilder::new(config).build().unwrap();
        let role = client.roles().default_role().clone();
        let registry1 = FunctionRegistry::new();
        let registry2 = FunctionRegistry::new();

        let exec_unit = AiExecUnit::new(client, role, registry1);
        let updated_unit = exec_unit.with_registry(registry2.clone());

        // 验证更新后的注册表
        assert_eq!(
            updated_unit.registry().get_functions().len(),
            registry2.get_functions().len()
        );
    }

    #[test]
    fn test_diagnostic_config_methods() {
        // 测试诊断配置相关方法
        let config = AiConfig::example();
        let client = AiClientBuilder::new(config).build().unwrap();
        let role = client.roles().default_role().clone();
        let registry = FunctionRegistry::new();

        let mut exec_unit = AiExecUnit::new(client, role, registry);
        
        // 测试初始状态
        assert!(exec_unit.diagnostic_config().is_none());
        
        // 测试设置诊断配置
        let diagnostic_config = DiagnosticConfig::standard();
        exec_unit.set_diagnostic_config(Some(diagnostic_config.clone()));
        assert_eq!(exec_unit.diagnostic_config(), &Some(diagnostic_config));
        
        // 测试清除诊断配置
        exec_unit.set_diagnostic_config(None);
        assert!(exec_unit.diagnostic_config().is_none());
    }
}
