use std::{collections::HashMap, sync::Arc};

use orion_error::{ToStructError, UvsLogicFrom};

use crate::{
    AiResult, FunctionCall, FunctionDefinition, FunctionResult, error::OrionAiReason,
    func::executor::FunctionExecutor,
};

/// 简化的函数注册表
#[derive(Clone, Default)]
pub struct FunctionRegistry {
    functions: HashMap<String, FunctionDefinition>,
    executors: HashMap<String, Arc<dyn FunctionExecutor>>,
}

impl FunctionRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册函数
    pub fn register_function(&mut self, function: FunctionDefinition) -> AiResult<()> {
        self.functions.insert(function.name.clone(), function);
        Ok(())
    }

    /// 注册执行器
    pub fn register_executor(
        &mut self,
        name: String,
        executor: Arc<dyn FunctionExecutor>,
    ) -> AiResult<()> {
        self.executors.insert(name, executor);
        Ok(())
    }

    /// 获取所有函数定义
    pub fn get_functions(&self) -> Vec<&FunctionDefinition> {
        self.functions.values().collect()
    }

    /// 获取指定函数的执行器
    pub fn get_executor(&self, function_name: &str) -> Option<Arc<dyn FunctionExecutor>> {
        self.executors.get(function_name).cloned()
    }

    /// 根据名称获取函数定义
    pub fn get_function(&self, name: &str) -> Option<&FunctionDefinition> {
        self.functions.get(name)
    }

    /// 执行函数调用
    pub async fn execute_function(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        let executor = self
            .executors
            .get(&function_call.function.name)
            .ok_or_else(|| OrionAiReason::from_logic("TODO: executor not found").to_err())?;

        executor.execute(function_call).await
    }

    /// 检查是否支持指定函数
    pub fn supports_function(&self, function_name: &str) -> bool {
        self.executors.contains_key(function_name)
    }

    /// 检查函数是否已存在
    pub fn contains_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// 获取所有支持的函数名称
    pub fn get_supported_function_names(&self) -> Vec<String> {
        self.executors.keys().cloned().collect()
    }

    /// 批量注册函数定义（优化版本）
    pub fn register_functions_batch(&mut self, functions: Vec<FunctionDefinition>) -> AiResult<()> {
        // 先检查所有函数名是否冲突
        for function in &functions {
            if self.functions.contains_key(&function.name) {
                return Err(OrionAiReason::from_logic(format!(
                    "Function '{}' already registered",
                    function.name
                ))
                .to_err());
            }
        }

        // 批量注册
        for function in functions {
            self.functions.insert(function.name.clone(), function);
        }

        Ok(())
    }

    /// 获取所有注册的函数名称
    pub fn get_function_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    /// 移除指定函数及其执行器
    pub fn unregister_function(&mut self, function_name: &str) -> bool {
        let function_removed = self.functions.remove(function_name).is_some();
        let executor_removed = self.executors.remove(function_name).is_some();
        function_removed || executor_removed
    }

    /// 批量注册函数
    pub fn register_functions(&mut self, functions: Vec<FunctionDefinition>) -> AiResult<()> {
        for function in functions {
            self.register_function(function)?;
        }
        Ok(())
    }

    /// 新增：克隆注册表
    pub fn clone_registry(&self) -> Self {
        let mut new_registry = Self::new();

        // 克隆函数定义
        for (name, function) in &self.functions {
            new_registry
                .functions
                .insert(name.clone(), function.clone());
        }

        // 克隆执行器引用（Arc 可以安全克隆）
        for (name, executor) in &self.executors {
            new_registry
                .executors
                .insert(name.clone(), executor.clone());
        }

        new_registry
    }

    /// 新增：获取所有函数定义的克隆
    pub fn clone_functions(&self) -> Vec<FunctionDefinition> {
        self.functions.values().cloned().collect()
    }
}
