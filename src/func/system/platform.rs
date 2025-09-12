use std::env;

/// 支持的操作系统平台
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    Unknown,
}

/// 检测当前运行平台
pub fn detect_platform() -> Platform {
    match env::consts::OS {
        "linux" => Platform::Linux,
        "macos" => Platform::MacOS,
        "windows" => Platform::Windows,
        _ => Platform::Unknown,
    }
}

/// 命令类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandType {
    Uptime,
    MemInfo,
    NetStat,
    Iostat,
    ProcessList,
}

/// 获取平台特定命令
pub fn get_platform_specific_command(command_type: CommandType, platform: &Platform) -> (String, Vec<&'static str>) {
    match command_type {
        CommandType::Uptime => match platform {
            Platform::MacOS => ("uptime".to_string(), vec![]),
            Platform::Linux => ("uptime".to_string(), vec![]),
            Platform::Windows => ("wmic".to_string(), vec!["os", "get", "lastbootuptime"]),
            Platform::Unknown => ("uptime".to_string(), vec![]),
        },
        CommandType::MemInfo => match platform {
            Platform::MacOS => ("vm_stat".to_string(), vec![]),
            Platform::Linux => ("free".to_string(), vec!["-h"]),
            Platform::Windows => ("wmic".to_string(), vec!["os", "get", "totalvisiblememorysize,freephysicalmemory"]),
            Platform::Unknown => ("vm_stat".to_string(), vec![]),
        },
        CommandType::NetStat => match platform {
            Platform::MacOS => ("netstat".to_string(), vec!["-an"]),
            Platform::Linux => ("netstat".to_string(), vec!["-tuln"]),
            Platform::Windows => ("netstat".to_string(), vec!["-an"]),
            Platform::Unknown => ("netstat".to_string(), vec!["-an"]),
        },
        CommandType::Iostat => match platform {
            Platform::MacOS => ("iostat".to_string(), vec![]),
            Platform::Linux => ("iostat".to_string(), vec![]),
            Platform::Windows => ("typeperf".to_string(), vec!["\\PhysicalDisk(*)\\Disk Read Bytes/sec", "\\PhysicalDisk(*)\\Disk Write Bytes/sec"]),
            Platform::Unknown => ("iostat".to_string(), vec![]),
        },
        CommandType::ProcessList => match platform {
            Platform::MacOS => ("ps".to_string(), vec![]),
            Platform::Linux => ("ps".to_string(), vec!["aux"]),
            Platform::Windows => ("tasklist".to_string(), vec![]),
            Platform::Unknown => ("ps".to_string(), vec!["aux"]),
        },
    }
}

/// 获取平台名称字符串
pub fn platform_name(platform: &Platform) -> &'static str {
    match platform {
        Platform::Linux => "Linux",
        Platform::MacOS => "macOS",
        Platform::Windows => "Windows",
        Platform::Unknown => "Unknown",
    }
}

/// 解析 vm_stat 输出（macOS 特定）
pub fn parse_vmstat_output(output: &str) -> serde_json::Value {
    use serde_json::json;
    
    let mut result = serde_json::Map::new();
    
    for line in output.lines() {
        if line.contains("page size of") {
            // 提取页面大小，处理格式如 "Mach Virtual Memory Statistics: (page size of 4096 bytes)"
            if let Some(size_str) = line.split("page size of").nth(1) {
                // 移除前导空格和可能的右括号，然后提取数字
                let cleaned = size_str.trim().trim_end_matches(')').trim();
                if let Some(size_num) = cleaned.split_whitespace().next() {
                    if let Ok(size) = size_num.parse::<u64>() {
                        result.insert("page_size".to_string(), json!(size));
                    }
                }
            }
        } else if let Some((key, value)) = line.split_once(':') {
            // 转换键名以匹配测试期望的格式
            let key_str = match key.trim() {
                "Pages free" => "free_count",
                "Pages active" => "active_count",
                "Pages inactive" => "inactive_count",
                "Pages wired" => "wired_count",
                k => &k.trim().replace(".", "_"),
            };
            
            if let Some(value_str) = value.trim().strip_suffix('.') {
                if let Ok(value_num) = value_str.trim().parse::<u64>() {
                    result.insert(key_str.to_string(), json!(value_num));
                }
            }
        }
    }
    
    // 计算可用内存（如果有可能）
    if let (Some(page_size), Some(free_count)) = (
        result.get("page_size").and_then(|v| v.as_u64()),
        result.get("free_count").and_then(|v| v.as_u64())
    ) {
        let free_bytes = page_size * free_count;
        result.insert("free_bytes".to_string(), json!(free_bytes));
    }
    
    json!(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_platform() {
        let platform = detect_platform();
        // 我们无法在测试中确定具体平台，但可以确保它返回一个有效的枚举值
        match platform {
            Platform::Linux | Platform::MacOS | Platform::Windows | Platform::Unknown => (),
        }
    }

    #[test]
    fn test_platform_name() {
        assert_eq!(platform_name(&Platform::Linux), "Linux");
        assert_eq!(platform_name(&Platform::MacOS), "macOS");
        assert_eq!(platform_name(&Platform::Windows), "Windows");
        assert_eq!(platform_name(&Platform::Unknown), "Unknown");
    }

    #[test]
    fn test_get_platform_specific_command() {
        // 测试 Linux 命令
        let (linux_uptime_cmd, linux_uptime_args) = get_platform_specific_command(CommandType::Uptime, &Platform::Linux);
        assert_eq!(linux_uptime_cmd, "uptime");
        assert_eq!(linux_uptime_args, vec![] as Vec<&str>);

        let (linux_meminfo_cmd, linux_meminfo_args) = get_platform_specific_command(CommandType::MemInfo, &Platform::Linux);
        assert_eq!(linux_meminfo_cmd, "free");
        assert_eq!(linux_meminfo_args, vec!["-h"]);

        let (linux_netstat_cmd, linux_netstat_args) = get_platform_specific_command(CommandType::NetStat, &Platform::Linux);
        assert_eq!(linux_netstat_cmd, "netstat");
        assert_eq!(linux_netstat_args, vec!["-tuln"]);

        // 测试 macOS 命令
        let (macos_uptime_cmd, macos_uptime_args) = get_platform_specific_command(CommandType::Uptime, &Platform::MacOS);
        assert_eq!(macos_uptime_cmd, "uptime");
        assert_eq!(macos_uptime_args, vec![] as Vec<&str>);

        let (macos_meminfo_cmd, macos_meminfo_args) = get_platform_specific_command(CommandType::MemInfo, &Platform::MacOS);
        assert_eq!(macos_meminfo_cmd, "vm_stat");
        assert_eq!(macos_meminfo_args, vec![] as Vec<&str>);

        let (macos_netstat_cmd, macos_netstat_args) = get_platform_specific_command(CommandType::NetStat, &Platform::MacOS);
        assert_eq!(macos_netstat_cmd, "netstat");
        assert_eq!(macos_netstat_args, vec!["-an"]);

        // 测试 Windows 命令
        let (windows_uptime_cmd, windows_uptime_args) = get_platform_specific_command(CommandType::Uptime, &Platform::Windows);
        assert_eq!(windows_uptime_cmd, "wmic");
        assert_eq!(windows_uptime_args, vec!["os", "get", "lastbootuptime"]);

        let (windows_meminfo_cmd, windows_meminfo_args) = get_platform_specific_command(CommandType::MemInfo, &Platform::Windows);
        assert_eq!(windows_meminfo_cmd, "wmic");
        assert_eq!(windows_meminfo_args, vec!["os", "get", "totalvisiblememorysize,freephysicalmemory"]);

        // 测试 Iostat 命令
        let (macos_iostat_cmd, macos_iostat_args) = get_platform_specific_command(CommandType::Iostat, &Platform::MacOS);
        assert_eq!(macos_iostat_cmd, "iostat");
        assert_eq!(macos_iostat_args, vec![] as Vec<&str>);

        let (windows_iostat_cmd, windows_iostat_args) = get_platform_specific_command(CommandType::Iostat, &Platform::Windows);
        assert_eq!(windows_iostat_cmd, "typeperf");
        assert_eq!(windows_iostat_args, vec!["\\PhysicalDisk(*)\\Disk Read Bytes/sec", "\\PhysicalDisk(*)\\Disk Write Bytes/sec"]);

        // 测试 ProcessList 命令
        let (macos_processlist_cmd, macos_processlist_args) = get_platform_specific_command(CommandType::ProcessList, &Platform::MacOS);
        assert_eq!(macos_processlist_cmd, "ps");
        assert_eq!(macos_processlist_args, vec![] as Vec<&str>);

        let (linux_processlist_cmd, linux_processlist_args) = get_platform_specific_command(CommandType::ProcessList, &Platform::Linux);
        assert_eq!(linux_processlist_cmd, "ps");
        assert_eq!(linux_processlist_args, vec!["aux"]);

        let (windows_processlist_cmd, windows_processlist_args) = get_platform_specific_command(CommandType::ProcessList, &Platform::Windows);
        assert_eq!(windows_processlist_cmd, "tasklist");
        assert_eq!(windows_processlist_args, vec![] as Vec<&str>);

        // 测试未知平台
        let (unknown_uptime_cmd, unknown_uptime_args) = get_platform_specific_command(CommandType::Uptime, &Platform::Unknown);
        assert_eq!(unknown_uptime_cmd, "uptime");
        assert_eq!(unknown_uptime_args, vec![] as Vec<&str>);
    }

    #[test]
    fn test_parse_vmstat_output() {
        let sample_output = r#"Mach Virtual Memory Statistics: (page size of 4096 bytes)
Pages free: 123456.
Pages active: 234567.
Pages inactive: 345678.
Pages wired: 456789.
"#;

        let parsed = parse_vmstat_output(sample_output);
        assert_eq!(parsed["page_size"], 4096);
        assert_eq!(parsed["free_count"], 123456);
        assert_eq!(parsed["active_count"], 234567);
        assert_eq!(parsed["inactive_count"], 345678);
        assert_eq!(parsed["wired_count"], 456789);
        assert_eq!(parsed["free_bytes"], 4096 * 123456);
    }
}