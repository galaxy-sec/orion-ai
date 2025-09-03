use std::sync::{Arc, OnceLock, RwLock};

use crate::{FunctionExecutor, func::registry::FunctionRegistry};

/// 全局函数注册表管理器
pub struct GlobalFunctionRegistry {
    // 使用 OnceLock 确保只初始化一次，并用RwLock支持重置
    global_registry: OnceLock<Arc<RwLock<Option<FunctionRegistry>>>>,
}

impl GlobalFunctionRegistry {
    /// 获取全局单例
    pub fn instance() -> &'static Self {
        static INSTANCE: GlobalFunctionRegistry = GlobalFunctionRegistry {
            global_registry: OnceLock::new(),
        };
        &INSTANCE
    }

    /// 初始化并注册所有工具（应用启动时调用）
    pub fn initialize() -> Result<(), orion_error::UvsReason> {
        let instance = Self::instance();

        // 如果尚未初始化，则创建新注册表
        if instance.global_registry.get().is_none() {
            let registry = Self::create_and_register_tools()?;
            let _ = instance
                .global_registry
                .set(Arc::new(RwLock::new(Some(registry))));
        }

        // 验证已注册
        if let Some(registry_arc) = instance.global_registry.get() {
            let registry_guard = registry_arc.read().unwrap();
            if registry_guard.is_none() {
                return Err(orion_error::UvsReason::validation_error(
                    "Registry initialization failed",
                ));
            }
        } else {
            return Err(orion_error::UvsReason::validation_error(
                "Global function registry not initialized. Call initialize() first.",
            ));
        }

        Ok(())
    }

    /// 重置全局注册表内容，仅清除已注册的工具（线程安全，主要用于测试）
    ///
    /// 注意：OnceLock 设计为只能设置一次，因此无法真正"重置"。
    /// 此方法将注册表内容设置为None，删除现有状态，
    /// 下次访问时重新初始化
    pub fn reset() {
        let instance = Self::instance();

        // 获取当前注册表并重置其内容
        if let Some(registry_arc) = instance.global_registry.get() {
            let mut registry = registry_arc.write().unwrap();
            *registry = None;
        }
    }

    /// 创建注册表并注册所有工具（硬编码）
    fn create_and_register_tools() -> Result<FunctionRegistry, orion_error::UvsReason> {
        let mut registry = FunctionRegistry::new();

        // 硬编码注册 Git 工具
        Self::register_git_tools(&mut registry)?;

        // 注册文件系统工具
        Self::register_filesystem_tools(&mut registry)?;

        // 注册系统信息工具
        Self::register_system_info_tools(&mut registry)?;

        // 注册网络工具
        Self::register_network_tools(&mut registry)?;

        Ok(registry)
    }

    /// 显式注册 Git 工具
    fn register_git_tools(registry: &mut FunctionRegistry) -> Result<(), orion_error::UvsReason> {
        use crate::func::git::{GitFunctionExecutor, create_git_functions};
        use std::sync::Arc;

        // 注册函数定义
        let git_functions = create_git_functions();
        for function in git_functions {
            registry.register_function(function).map_err(|e| {
                orion_error::UvsReason::validation_error(format!(
                    "Failed to register git function: {}",
                    e
                ))
            })?;
        }

        // 注册执行器
        let git_executor = Arc::new(GitFunctionExecutor);
        for function_name in git_executor.supported_functions() {
            registry
                .register_executor(function_name, git_executor.clone())
                .map_err(|e| {
                    orion_error::UvsReason::validation_error(format!(
                        "Failed to register git executor: {}",
                        e
                    ))
                })?;
        }

        Ok(())
    }

    /// 注册文件系统工具
    fn register_filesystem_tools(
        registry: &mut FunctionRegistry,
    ) -> Result<(), orion_error::UvsReason> {
        use crate::func::system::{FileSystemExecutor, create_fs_functions};
        use std::sync::Arc;

        let fs_functions = create_fs_functions();
        for function in fs_functions {
            registry.register_function(function).map_err(|e| {
                orion_error::UvsReason::validation_error(format!(
                    "Failed to register filesystem function: {}",
                    e
                ))
            })?;
        }

        let fs_executor = Arc::new(FileSystemExecutor);
        for function_name in fs_executor.supported_functions() {
            registry
                .register_executor(function_name, fs_executor.clone())
                .map_err(|e| {
                    orion_error::UvsReason::validation_error(format!(
                        "Failed to register filesystem executor: {}",
                        e
                    ))
                })?;
        }

        Ok(())
    }

    /// 注册系统信息工具
    fn register_system_info_tools(
        registry: &mut FunctionRegistry,
    ) -> Result<(), orion_error::UvsReason> {
        use crate::func::system::{SystemInfoExecutor, create_sys_functions};
        use std::sync::Arc;

        let sys_functions = create_sys_functions();
        for function in sys_functions {
            registry.register_function(function).map_err(|e| {
                orion_error::UvsReason::validation_error(format!(
                    "Failed to register system info function: {}",
                    e
                ))
            })?;
        }

        let sys_executor = Arc::new(SystemInfoExecutor);
        for function_name in sys_executor.supported_functions() {
            registry
                .register_executor(function_name, sys_executor.clone())
                .map_err(|e| {
                    orion_error::UvsReason::validation_error(format!(
                        "Failed to register system info executor: {}",
                        e
                    ))
                })?;
        }

        Ok(())
    }

    /// 注册网络工具
    fn register_network_tools(
        registry: &mut FunctionRegistry,
    ) -> Result<(), orion_error::UvsReason> {
        use crate::func::system::{NetworkExecutor, create_net_functions};
        use std::sync::Arc;

        let net_functions = create_net_functions();
        for function in net_functions {
            registry.register_function(function).map_err(|e| {
                orion_error::UvsReason::validation_error(format!(
                    "Failed to register network function: {}",
                    e
                ))
            })?;
        }

        let net_executor = Arc::new(NetworkExecutor);
        for function_name in net_executor.supported_functions() {
            registry
                .register_executor(function_name, net_executor.clone())
                .map_err(|e| {
                    orion_error::UvsReason::validation_error(format!(
                        "Failed to register network executor: {}",
                        e
                    ))
                })?;
        }

        Ok(())
    }

    /// 获取注册表的克隆副本（避免锁竞争）
    pub fn get_registry() -> Result<FunctionRegistry, orion_error::UvsReason> {
        // 确保注册表已初始化（自动初始化）
        let instance = Self::instance();

        if let Some(registry_arc) = instance.global_registry.get() {
            let registry_guard = registry_arc.read().unwrap();
            match registry_guard.as_ref() {
                Some(registry) => Ok(registry.clone_registry()),
                None => Err(orion_error::UvsReason::validation_error(
                    "Global function registry not initialized. Call initialize() first.",
                )),
            }
        } else {
            // 注册表从未初始化
            Err(orion_error::UvsReason::validation_error(
                "Global function registry not initialized. Call initialize() first.",
            ))
        }
    }

    /// 🎯 获取注册表的克隆副本，并根据指定工具列表进行过滤
    pub fn get_registry_with_tools(
        tools: &[String],
    ) -> Result<FunctionRegistry, orion_error::UvsReason> {
        // 首先获取完整的注册表副本
        let full_registry = Self::get_registry()?;

        // 如果工具列表为空，返回所有函数
        if tools.is_empty() {
            return Ok(full_registry);
        }

        // 否则，过滤出指定工具的函数
        let filtered_functions = full_registry
            .get_functions()
            .into_iter()
            .filter(|func_def| tools.contains(&func_def.name))
            .cloned()
            .collect::<Vec<_>>();

        // 创建新的注册表并只包含过滤后的函数
        let mut filtered_registry = FunctionRegistry::new();
        for function_def in filtered_functions {
            filtered_registry
                .register_function(function_def)
                .map_err(|e| {
                    orion_error::UvsReason::validation_error(format!(
                        "Failed to register filtered function: {}",
                        e
                    ))
                })?;
        }

        // 复制执行器引用
        for tool_name in tools {
            if let Some(executor) = full_registry.get_executor(tool_name) {
                filtered_registry
                    .register_executor(tool_name.clone(), executor)
                    .map_err(|e| {
                        orion_error::UvsReason::validation_error(format!(
                            "Failed to register executor for {}: {}",
                            tool_name, e
                        ))
                    })?;
            }
        }

        Ok(filtered_registry)
    }
}

#[cfg(test)]
mod global_registry_tests {
    // 添加测试用例来验证 get_registry_with_tools 功能
    #[tokio::test]
    async fn test_get_registry_with_tools() {
        // 确保注册表已初始化（不重置以避免并发问题）
        GlobalFunctionRegistry::initialize().unwrap();

        // 测试指定工具列表
        let tools = vec!["git-status".to_string(), "git-add".to_string()];
        let filtered_registry = GlobalFunctionRegistry::get_registry_with_tools(&tools).unwrap();

        let filtered_functions = filtered_registry.get_supported_function_names();
        assert_eq!(filtered_functions.len(), 2);
        assert!(filtered_functions.contains(&"git-status".to_string()));
        assert!(filtered_functions.contains(&"git-add".to_string()));
        assert!(!filtered_functions.contains(&"git-commit".to_string()));

        // 测试单个工具
        let single_tool = vec!["git-status".to_string()];
        let single_registry =
            GlobalFunctionRegistry::get_registry_with_tools(&single_tool).unwrap();

        let single_functions = single_registry.get_supported_function_names();
        assert_eq!(single_functions.len(), 1);
        assert!(single_functions.contains(&"git-status".to_string()));

        // 测试空工具列表（应该返回所有工具）
        let empty_tools: Vec<String> = vec![];
        let full_registry = GlobalFunctionRegistry::get_registry_with_tools(&empty_tools).unwrap();

        let full_functions = full_registry.get_supported_function_names();
        let all_registry = GlobalFunctionRegistry::get_registry().unwrap();
        let all_functions = all_registry.get_supported_function_names();

        assert_eq!(full_functions.len(), all_functions.len());
        for func_name in &full_functions {
            assert!(all_functions.contains(func_name));
        }

        // 测试不存在的工具
        let nonexistent_tools = vec!["nonexistent_tool".to_string()];
        let empty_registry =
            GlobalFunctionRegistry::get_registry_with_tools(&nonexistent_tools).unwrap();

        let empty_functions = empty_registry.get_supported_function_names();
        assert_eq!(empty_functions.len(), 0);

        // 测试混合存在的和不存在的工具
        let mixed_tools = vec!["git-status".to_string(), "nonexistent_tool".to_string()];
        let mixed_registry = GlobalFunctionRegistry::get_registry_with_tools(&mixed_tools).unwrap();

        let mixed_functions = mixed_registry.get_supported_function_names();
        assert_eq!(mixed_functions.len(), 1);
        assert!(mixed_functions.contains(&"git-status".to_string()));
    }
    use super::*;

    #[tokio::test]
    async fn test_global_registry_initialization() {
        // 确保注册表初始化，不重置
        assert!(GlobalFunctionRegistry::initialize().is_ok());

        // 获取注册表副本
        let registry = GlobalFunctionRegistry::get_registry();
        assert!(registry.is_ok());

        let registry = registry.unwrap();
        let function_names = registry.get_supported_function_names();

        // 验证Git工具已注册
        assert!(function_names.contains(&"git-status".to_string()));
        assert!(function_names.contains(&"git-commit".to_string()));
        assert!(function_names.contains(&"git-add".to_string()));
        assert!(function_names.contains(&"git-push".to_string()));
        assert!(function_names.contains(&"git-diff".to_string()));

        // 验证新的系统命令工具已注册
        assert!(function_names.contains(&"fs-ls".to_string()));
        assert!(function_names.contains(&"fs-pwd".to_string()));
        assert!(function_names.contains(&"fs-cat".to_string()));
        assert!(function_names.contains(&"fs-find".to_string()));

        assert!(function_names.contains(&"sys-uname".to_string()));
        assert!(function_names.contains(&"sys-ps".to_string()));
        assert!(function_names.contains(&"sys-df".to_string()));

        assert!(function_names.contains(&"net-ping".to_string()));
    }

    #[tokio::test]
    async fn test_registry_cloning() {
        // 确保注册表初始化
        assert!(GlobalFunctionRegistry::initialize().is_ok());

        // 获取第一个副本
        let registry1 = GlobalFunctionRegistry::get_registry().unwrap();
        let function_names1 = registry1.get_supported_function_names();

        // 获取第二个副本
        let registry2 = GlobalFunctionRegistry::get_registry().unwrap();
        let function_names2 = registry2.get_supported_function_names();

        // 验证两个副本包含相同的函数（不考虑顺序）
        assert_eq!(function_names1.len(), function_names2.len());
        for function_name in &function_names1 {
            assert!(function_names2.contains(function_name));
        }
        for function_name in &function_names2 {
            assert!(function_names1.contains(function_name));
        }
    }

    #[tokio::test]
    async fn test_double_initialization() {
        // 确保注册表初始化
        assert!(GlobalFunctionRegistry::initialize().is_ok());

        // 第二次初始化应该不会失败
        assert!(GlobalFunctionRegistry::initialize().is_ok());

        // 注册表应该仍然可用
        let registry = GlobalFunctionRegistry::get_registry();
        assert!(registry.is_ok());
    }
}
