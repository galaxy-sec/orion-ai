//! AI驱动的系统诊断示例程序
//! 
//! 这个示例程序展示了如何使用 Orion AI 的 operations 角色进行AI驱动的系统诊断。
//! 通过AI推理分析系统状态，提供智能化的诊断建议和问题解决方案。

use orion_ai::*;
use orion_variate::vars::EnvDict;
use orion_ai::types::DiagnosticDepth;

#[tokio::main]
async fn main() -> AiResult<()> {
    orion_ai::infra::once_init_log();
    // 初始化全局函数注册表
    GlobalFunctionRegistry::initialize()?;

    // 创建环境变量字典
    let dict = EnvDict::new();

    println!("=== AI驱动的系统诊断示例 ===\n");
    println!("使用 operations 角色进行智能系统诊断\n");

    // 示例1: AI对话式系统诊断
    println!("1. AI对话式系统诊断:");
    ai_driven_system_diagnosis(&dict).await?;

    // 示例2: 交互式问题排查
    println!("\n2. 交互式问题排查:");
    interactive_troubleshooting(&dict).await?;

    // 示例3: 智能性能优化建议
    println!("\n3. 智能性能优化建议:");
    intelligent_performance_analysis(&dict).await?;

    // 示例4: 错误信息展示
    println!("\n4. 系统诊断错误信息展示:");
    demonstrate_error_scenarios(&dict).await?;

    Ok(())
}

/// 错误信息展示 - 演示各种系统诊断中的错误场景
async fn demonstrate_error_scenarios(dict: &EnvDict) -> AiResult<()> {
    let exec_unit = AiExecUnitBuilder::new(dict.clone())
        .with_role("operations")
        .with_diagnostic_depth(DiagnosticDepth::Advanced)
        .build()?;

    // 场景1: 权限不足错误
    println!("场景1: 权限不足错误");
    let permission_error_prompt = r#"
    请尝试执行需要root权限的系统诊断命令，例如：
    - 读取系统日志文件: /var/log/system.log
    - 访问进程信息: /proc/1/status
    - 检查系统配置: /etc/sudoers
    
    故意触发权限错误，然后详细解释：
    1. 具体的错误信息和错误代码
    2. 为什么会出现这个错误
    3. 如何通过sudo或修改权限解决
    4. 替代的诊断方法（如果适用）
    
    请展示真实的错误输出和解决方案。
    "#;

    match exec_unit.execute_with_func(permission_error_prompt).await {
        Ok(response) => println!("权限错误处理:\n{}", response.content),
        Err(e) => println!("捕获到权限错误: {:?}", e),
    }

    // 场景2: 文件不存在错误
    println!("\n场景2: 文件或目录不存在错误");
    let file_not_found_prompt = r#"
    请尝试访问以下不存在的文件和目录：
    - /tmp/non_existent_config.conf
    - /var/log/fake_application.log
    - /home/nonexistent_user/.bashrc
    
    展示这些操作产生的错误信息，并解释：
    1. ENOENT (No such file or directory) 错误的具体含义
    2. 如何验证文件是否真的不存在
    3. 可能的替代文件路径
    4. 创建缺失文件或目录的方法
    5. 检查文件系统权限和所有权
    
    提供实用的排查步骤。
    "#;

    match exec_unit.execute_with_func(file_not_found_prompt).await {
        Ok(response) => println!("文件不存在错误处理:\n{}", response.content),
        Err(e) => println!("捕获到文件错误: {:?}", e),
    }

    // 场景3: 网络连接错误
    println!("\n场景3: 网络连接诊断错误");
    let network_error_prompt = r#"
    请执行网络连接诊断，故意触发以下错误：
    - 连接到不存在的域名: nonexistent-server-12345.com
    - 连接到未开放的端口: localhost:99999
    - DNS解析失败: invalid.hostname.local
    
    展示这些网络错误的详细信息：
    1. 连接超时错误 (Connection timeout)
    2. DNS解析错误 (Name or service not known)
    3. 连接被拒绝 (Connection refused)
    4. 网络不可达 (Network is unreachable)
    
    解释每个错误的含义和排查方法：
    - 检查网络连接状态
    - 验证DNS配置
    - 测试端口连通性
    - 检查防火墙设置
    - 使用替代的诊断工具
    
    提供网络故障排除的完整流程。
    "#;

    match exec_unit.execute_with_func(network_error_prompt).await {
        Ok(response) => println!("网络错误处理:\n{}", response.content),
        Err(e) => println!("捕获到网络错误: {:?}", e),
    }

    // 场景4: 命令执行错误
    println!("\n场景4: 命令执行错误");
    let command_error_prompt = r#"
    请尝试执行以下会失败的系统命令：
    - 运行不存在的命令: nonexistent_system_command
    - 使用无效的参数: ls --invalid-parameter
    - 执行权限不足的二进制: /etc/passwd
    - 语法错误的shell命令: "echo "unclosed quote"
    
    展示这些命令执行错误的完整输出：
    1. "command not found" 错误
    2. "permission denied" 错误
    3. "invalid option" 错误
    4. shell语法错误
    
    分析每个错误类型：
    - 如何验证命令是否存在
    - 检查命令路径和PATH环境变量
    - 验证文件权限和可执行性
    - 检查命令语法和参数
    - 使用which/type命令定位问题
    
    提供命令故障排除的实用技巧。
    "#;

    match exec_unit.execute_with_func(command_error_prompt).await {
        Ok(response) => println!("命令错误处理:\n{}", response.content),
        Err(e) => println!("捕获到命令错误: {:?}", e),
    }

    // 场景5: 资源不足错误
    println!("\n场景5: 系统资源不足错误");
    let resource_error_prompt = r#"
    请诊断以下资源不足的情况：
    - 磁盘空间不足 (模拟df -h显示100%使用)
    - 内存不足 (模拟free -h显示高使用率)
    - 文件描述符耗尽
    - 进程数达到限制
    - 网络端口耗尽
    
    展示这些资源错误的详细信息：
    1. "No space left on device" 错误
    2. "Out of memory" 或内存分配失败
    3. "Too many open files" 错误
    4. "Resource temporarily unavailable" 错误
    
    提供资源问题的解决方案：
    - 清理磁盘空间的具体步骤
    - 识别内存泄漏的进程
    - 调整系统限制 (ulimit)
    - 优化资源使用
    - 监控系统资源
    
    给出预防和监控建议。
    "#;

    match exec_unit.execute_with_func(resource_error_prompt).await {
        Ok(response) => println!("资源错误处理:\n{}", response.content),
        Err(e) => println!("捕获到资源错误: {:?}", e),
    }

    println!("\n=== 错误信息展示完成 ===");
    println!("所有错误场景已演示，请参考AI提供的具体解决方案。");

    Ok(())
}

/// AI驱动的系统诊断 - 使用operations角色进行智能分析
async fn ai_driven_system_diagnosis(dict: &EnvDict) -> AiResult<()> {
    let exec_unit = AiExecUnitBuilder::new(dict.clone())
        .with_role("operations")
        .with_diagnostic_depth(DiagnosticDepth::Standard)
        .build()?;

    // 构建AI诊断提示
    let diagnostic_prompt = r#"
    请作为系统诊断专家，执行以下任务：
    
    1. 收集系统基础信息（CPU、内存、磁盘、网络）
    2. 分析当前系统性能状态
    3. 识别潜在的性能瓶颈
    4. 提供具体的优化建议
    5. 预测可能的问题风险
    
    请以结构化的方式提供诊断结果，包括：
    - 系统健康评分（0-100）
    - 主要发现的问题
    - 详细的性能分析
    - 优先级排序的改进建议
    - 监控和预警建议
    
    请使用专业的系统管理员视角，提供实用且可操作的建议。
    "#;

    let response = exec_unit.execute_with_func(diagnostic_prompt).await?;
    println!("AI诊断结果:\n{}", response.content);

    Ok(())
}

/// 交互式问题排查 - 用户可与AI对话解决问题
async fn interactive_troubleshooting(dict: &EnvDict) -> AiResult<()> {
    let exec_unit = AiExecUnitBuilder::new(dict.clone())
        .with_role("operations")
        .with_diagnostic_depth(DiagnosticDepth::Advanced)
        .build()?;

    // 模拟用户遇到系统缓慢问题的交互场景
    let user_problem = "我的系统最近变得很慢，启动程序需要很长时间，请帮我分析原因";
    
    let troubleshooting_prompt = format!(r#"
    用户问题：{}
    
    请执行以下诊断流程：
    
    1. 首先收集系统运行时间和负载信息
    2. 检查内存使用情况和可能的内存泄漏
    3. 分析磁盘I/O性能，检查是否有磁盘瓶颈
    4. 查看高CPU消耗的进程
    5. 检查网络连接状态
    
    基于收集到的信息，请：
    - 分析导致系统缓慢的可能原因
    - 提供逐步的排查指导
    - 给出具体的解决建议
    - 说明如何预防类似问题
    
    请用通俗易懂的语言解释技术细节。
    "#, user_problem);

    let response = exec_unit.execute_with_func(&troubleshooting_prompt).await?;
    println!("问题排查结果:\n{}", response.content);

    Ok(())
}

/// 智能性能优化建议 - 基于AI分析提供个性化优化方案
async fn intelligent_performance_analysis(dict: &EnvDict) -> AiResult<()> {
    let exec_unit = AiExecUnitBuilder::new(dict.clone())
        .with_role("operations")
        .with_diagnostic_depth(DiagnosticDepth::Advanced)
        .build()?;

    // 构建性能分析提示，要求AI提供个性化优化方案
    let performance_prompt = r#"
    请作为性能优化专家，执行深度系统分析并提供个性化优化方案：
    
    诊断任务：
    1. 收集详细的系统性能指标
    2. 识别资源使用的瓶颈点
    3. 分析系统配置的优化空间
    4. 评估当前运行的进程和服务
    5. 检查系统启动项和后台服务
    
    输出要求：
    - 系统性能总览（CPU/内存/磁盘/网络）
    - 具体的性能瓶颈识别
    - 针对当前系统的个性化优化建议
    - 优先级排序的改进措施
    - 长期维护建议
    - 监控关键指标的方法
    
    请提供可立即执行的优化命令和配置调整建议。
    "#;

    let response = exec_unit.execute_with_func(performance_prompt).await?;
    println!("性能优化建议:\n{}", response.content);

    Ok(())
}