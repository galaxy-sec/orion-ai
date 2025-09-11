//! 进程数据解析器
//!
//! 此模块提供了用于解析系统进程信息的解析器实现

use crate::exec_unit::parsers::{ParseError, ParseResult, ParserTrait, ParsedData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 进程信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// 进程ID
    pub pid: u32,
    /// 进程名称
    pub name: String,
    /// 进程状态
    pub status: String,
    /// CPU使用率
    pub cpu_usage: f32,
    /// 内存使用量（KB）
    pub memory_usage: u64,
    /// 父进程ID
    pub parent_pid: Option<u32>,
    /// 进程启动时间
    pub start_time: Option<String>,
    /// 进程命令行
    pub command_line: Option<String>,
    /// 进程用户
    pub user: Option<String>,
    /// 额外属性
    pub extra: HashMap<String, String>,
}

impl ParsedData for ProcessInfo {
    fn data_type(&self) -> &str {
        "process_info"
    }

    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }
}

/// 进程列表结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessList {
    /// 进程列表
    pub processes: Vec<ProcessInfo>,
    /// 总进程数
    pub total_count: usize,
    /// 系统信息
    pub system_info: HashMap<String, String>,
}

impl ParsedData for ProcessList {
    fn data_type(&self) -> &str {
        "process_list"
    }

    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }
}

/// 进程数据解析器
#[derive(Debug)]
#[allow(dead_code)]
pub struct ProcessParser {
    /// 解析器名称
    name: String,
    /// 支持的格式
    supported_formats: Vec<String>,
}

impl ProcessParser {
    /// 创建新的进程解析器
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            name: "process".to_string(),
            supported_formats: vec![
                "ps".to_string(),
                "top".to_string(),
                "htop".to_string(),
                "procfs".to_string(),
            ],
        }
    }

    /// 解析ps命令输出
    #[allow(dead_code)]
    fn parse_ps_output(&self, output: &str) -> ParseResult<Box<dyn ParsedData>> {
        let lines: Vec<&str> = output.lines().collect();
        if lines.is_empty() {
            return Err(ParseError::InvalidFormat("Empty ps output".to_string()));
        }

        // 跳过标题行
        let data_lines = &lines[1..];
        let mut processes = Vec::new();

        for line in data_lines {
            if line.trim().is_empty() {
                continue;
            }

            // 简单解析ps输出格式
            // 假设格式为: PID TTY TIME CMD
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                continue;
            }

            let pid = parts[0]
                .parse::<u32>()
                .map_err(ParseError::ParseIntError)?;
            
            let name = parts[3..].join(" ");
            let command_line = Some(name.clone());
            
            let process_info = ProcessInfo {
                pid,
                name,
                status: "R".to_string(), // 默认状态为运行中
                cpu_usage: 0.0,         // ps输出不包含CPU使用率
                memory_usage: 0,        // ps输出不包含内存使用量
                parent_pid: None,
                start_time: None,
                command_line,
                user: None,
                extra: HashMap::new(),
            };

            processes.push(process_info);
        }

        let total_count = processes.len();
        let process_list = ProcessList {
            processes,
            total_count,
            system_info: HashMap::new(),
        };

        Ok(Box::new(process_list))
    }

    /// 解析top命令输出
    #[allow(dead_code)]
    fn parse_top_output(&self, output: &str) -> ParseResult<Box<dyn ParsedData>> {
        let lines: Vec<&str> = output.lines().collect();
        if lines.len() < 2 {
            return Err(ParseError::InvalidFormat("Incomplete top output".to_string()));
        }

        let mut processes = Vec::new();
        let mut process_section = false;

        for line in lines {
            if line.starts_with("PID") {
                process_section = true;
                continue;
            }

            if process_section && !line.trim().is_empty() {
                // 简单解析top输出格式
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 12 {
                    let pid = parts[0]
                        .parse::<u32>()
                        .map_err(ParseError::ParseIntError)?;
                    
                    let user = parts[1].to_string();
                    let cpu_usage = parts[8]
                        .parse::<f32>()
                        .map_err(ParseError::ParseFloatError)?;
                    
                    // 修复：将memory_usage解析为f64然后转换为u64
                    let memory_usage = parts[9]
                        .parse::<f64>()
                        .map_err(ParseError::ParseFloatError)?;
                    
                    let command = parts[11..].join(" ");

                    let process_info = ProcessInfo {
                        pid,
                        name: command.clone(),
                        status: "R".to_string(), // 默认状态为运行中
                        cpu_usage,
                        memory_usage: memory_usage as u64,
                        parent_pid: None,
                        start_time: None,
                        command_line: Some(command),
                        user: Some(user),
                        extra: HashMap::new(),
                    };

                    processes.push(process_info);
                }
            }
        }

        let total_count = processes.len();
            let process_list = ProcessList {
                processes,
                total_count,
                system_info: HashMap::new(),
            };

        Ok(Box::new(process_list))
    }

    /// 解析/proc文件系统数据
    #[allow(dead_code)]
    fn parse_procfs_data(&self, data: &str) -> ParseResult<Box<dyn ParsedData>> {
        // 这里简化处理，实际实现需要解析/proc/[pid]/stat等文件
        let lines: Vec<&str> = data.lines().collect();
        if lines.is_empty() {
            return Err(ParseError::InvalidFormat("Empty procfs data".to_string()));
        }

        let mut processes = Vec::new();

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            // 简化解析/proc/[pid]/stat格式
            // 实际格式更复杂，这里仅作示例
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let pid = parts[0]
                    .parse::<u32>()
                    .map_err(ParseError::ParseIntError)?;
                
                let name = parts[1].trim_matches('(').trim_matches(')').to_string();
                let status = parts[2].to_string();
                
                let process_info = ProcessInfo {
                    pid,
                    name,
                    status,
                    cpu_usage: 0.0,  // 需要计算
                    memory_usage: 0, // 需要从其他文件获取
                    parent_pid: None,
                    start_time: None,
                    command_line: None,
                    user: None,
                    extra: HashMap::new(),
                };

                processes.push(process_info);
            }
        }

        let total_count = processes.len();
            let process_list = ProcessList {
                processes,
                total_count,
                system_info: HashMap::new(),
            };

        Ok(Box::new(process_list))
    }
}

impl ParserTrait for ProcessParser {
    fn parse(&self, input: &str) -> ParseResult<Box<dyn ParsedData>> {
        // 尝试确定输入格式并选择相应的解析方法
        if input.contains("PID") && input.contains("TTY") && input.contains("TIME") {
            // 可能是ps命令输出
            self.parse_ps_output(input)
        } else if input.contains("PID") && input.contains("USER") && input.contains("%CPU") {
            // 可能是top命令输出
            self.parse_top_output(input)
        } else if input.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            // 可能是/proc文件系统数据
            self.parse_procfs_data(input)
        } else {
            Err(ParseError::InvalidFormat(
                "Unable to determine process data format".to_string(),
            ))
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn can_parse(&self, input: &str) -> bool {
        // 检查输入是否包含进程数据的特征
        input.contains("PID") || 
        input.chars().next().is_some_and(|c| c.is_ascii_digit()) ||
        (input.lines().any(|line| line.split_whitespace().count() > 3) && 
         (input.contains("USER") || input.contains("TTY") || input.contains("CMD") || input.contains("COMMAND")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_parser_creation() {
        let parser = ProcessParser::new();
        assert_eq!(parser.name(), "process");
    }

    #[test]
    fn test_can_parse() {
        let parser = ProcessParser::new();
        
        // 测试ps输出
        let ps_output = "PID TTY TIME CMD\n1234 pts/0 00:00:01 bash";
        assert!(parser.can_parse(ps_output));
        
        // 测试top输出
        let top_output = "PID USER PR NI VIRT RES SHR S %CPU %MEM TIME+ COMMAND\n1234 user 20 0 12345 6789 0 S 0.0 0.1 0:00.01 bash";
        assert!(parser.can_parse(top_output));
        
        // 测试procfs数据
        let procfs_data = "1234 (bash) S 5678 1234 1234 0 -1 4194560 1542 0 0 0 0 0 0 0 20 0 1 0 12345678 1234567 123 18446744073709551615 1 1 0 0 0 0 0 0 0 17 3 0 0 0 0 0";
        assert!(parser.can_parse(procfs_data));
        
        // 测试无效数据
        let invalid_data = "This is not process data";
        assert!(!parser.can_parse(invalid_data));
    }

    #[test]
    fn test_parse_ps_output() {
        let parser = ProcessParser::new();
        let ps_output = "PID TTY TIME CMD\n1234 pts/0 00:00:01 bash\n5678 pts/1 00:00:02 vim";
        
        let result = parser.parse_ps_output(ps_output);
        assert!(result.is_ok());
        
        let process_list = result.unwrap();
        let process_list = process_list.as_any_ref().downcast_ref::<ProcessList>().unwrap();
        
        assert_eq!(process_list.total_count, 2);
        assert_eq!(process_list.processes[0].pid, 1234);
        assert_eq!(process_list.processes[0].name, "bash");
        assert_eq!(process_list.processes[1].pid, 5678);
        assert_eq!(process_list.processes[1].name, "vim");
    }

    #[test]
    fn test_parse_top_output() {
        let parser = ProcessParser::new();
        let top_output = "PID USER PR NI VIRT RES SHR S %CPU %MEM TIME+ COMMAND\n1234 user 20 0 12345 6789 0 S 0.0 0.1 0:00.01 bash\n5678 user 20 0 23456 7890 0 S 0.1 0.2 0:00.02 vim";
        
        let result = parser.parse_top_output(top_output);
        assert!(result.is_ok());
        
        let process_list = result.unwrap();
        let process_list = process_list.as_any_ref().downcast_ref::<ProcessList>().unwrap();
        
        assert_eq!(process_list.total_count, 2);
        assert_eq!(process_list.processes[0].pid, 1234);
        assert_eq!(process_list.processes[0].cpu_usage, 0.0);
        assert_eq!(process_list.processes[1].pid, 5678);
        assert_eq!(process_list.processes[1].cpu_usage, 0.1);
    }

    #[test]
    fn test_process_info_data_type() {
        let process_info = ProcessInfo {
            pid: 1234,
            name: "bash".to_string(),
            status: "R".to_string(),
            cpu_usage: 0.5,
            memory_usage: 1024,
            parent_pid: Some(1),
            start_time: Some("2023-01-01 10:00:00".to_string()),
            command_line: Some("/bin/bash".to_string()),
            user: Some("user".to_string()),
            extra: HashMap::new(),
        };
        
        assert_eq!(process_info.data_type(), "process_info");
        
        let json = process_info.to_json();
        assert!(json.is_ok());
        assert!(json.unwrap().contains("\"pid\":1234"));
    }

    #[test]
    fn test_process_list_data_type() {
        let processes = vec![ProcessInfo {
            pid: 1234,
            name: "bash".to_string(),
            status: "R".to_string(),
            cpu_usage: 0.5,
            memory_usage: 1024,
            parent_pid: None,
            start_time: None,
            command_line: None,
            user: None,
            extra: HashMap::new(),
        }];
        
        let process_list = ProcessList {
            processes,
            total_count: 1,
            system_info: HashMap::new(),
        };
        
        assert_eq!(process_list.data_type(), "process_list");
        
        let json = process_list.to_json();
        assert!(json.is_ok());
        assert!(json.unwrap().contains("\"total_count\":1"));
    }
}