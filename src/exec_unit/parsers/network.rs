//! 网络数据解析器
//! 
//! 该模块提供了用于解析网络相关命令输出的解析器，
//! 支持网络连接、接口统计和路由表数据的解析。

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::exec_unit::parsers::{
    ParseError, ParseResult, ParserTrait, ParsedData,
};

/// 网络连接信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    /// 协议类型 (TCP, UDP, etc.)
    pub protocol: String,
    /// 本地地址和端口
    pub local_address: String,
    /// 远程地址和端口
    pub remote_address: String,
    /// 连接状态
    pub state: String,
    /// 进程ID (可选)
    pub pid: Option<u32>,
    /// 进程名称 (可选)
    pub process_name: Option<String>,
    /// 额外信息
    pub extra: HashMap<String, Value>,
}

/// 网络接口统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// 接口名称
    pub name: String,
    /// 接口状态 (UP, DOWN, etc.)
    pub status: String,
    /// MTU大小
    pub mtu: u32,
    /// 接收字节数
    pub rx_bytes: u64,
    /// 接收数据包数
    pub rx_packets: u64,
    /// 接收错误数
    pub rx_errors: u64,
    /// 发送字节数
    pub tx_bytes: u64,
    /// 发送数据包数
    pub tx_packets: u64,
    /// 发送错误数
    pub tx_errors: u64,
    /// IP地址 (可选)
    pub ip_address: Option<String>,
    /// MAC地址 (可选)
    pub mac_address: Option<String>,
    /// 额外信息
    pub extra: HashMap<String, Value>,
}

/// 路由表条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    /// 目标网络
    pub destination: String,
    /// 网关
    pub gateway: String,
    /// 子网掩码
    pub netmask: String,
    /// 接口名称
    pub interface: String,
    /// 路由标志
    pub flags: String,
    /// 度量值
    pub metric: Option<u32>,
    /// 额外信息
    pub extra: HashMap<String, Value>,
}

/// 网络连接列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnectionList {
    /// 连接列表
    pub connections: Vec<NetworkConnection>,
    /// 总连接数
    pub total_count: usize,
    /// 按协议分组的连接数
    pub protocol_counts: HashMap<String, usize>,
    /// 按状态分组的连接数
    pub state_counts: HashMap<String, usize>,
    /// 系统信息
    pub system_info: HashMap<String, Value>,
}

/// 网络接口列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceList {
    /// 接口列表
    pub interfaces: Vec<NetworkInterface>,
    /// 总接口数
    pub total_count: usize,
    /// 总接收字节数
    pub total_rx_bytes: u64,
    /// 总发送字节数
    pub total_tx_bytes: u64,
    /// 系统信息
    pub system_info: HashMap<String, Value>,
}

/// 路由表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingTable {
    /// 路由条目列表
    pub routes: Vec<RouteEntry>,
    /// 总路由数
    pub total_count: usize,
    /// 默认网关 (可选)
    pub default_gateway: Option<String>,
    /// 系统信息
    pub system_info: HashMap<String, Value>,
}

/// 网络数据解析器
#[derive(Debug)]
#[allow(dead_code)]
pub struct NetworkParser {
    name: String,
}

impl NetworkParser {
    /// 创建新的网络数据解析器
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            name: "network".to_string(),
        }
    }

    /// 解析netstat命令输出
    fn parse_netstat_output(&self, data: &str) -> ParseResult<Box<dyn ParsedData>> {
        let lines: Vec<&str> = data.lines().collect();
        if lines.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let mut connections = Vec::new();
        let mut protocol_counts = HashMap::new();
        let mut state_counts = HashMap::new();

        // 跳过标题行
        let start_idx = lines.iter()
            .position(|line| line.contains("Proto") || line.contains("Local Address"))
            .unwrap_or(0);

        for line in lines.iter().skip(start_idx + 1) {
            if line.trim().is_empty() {
                continue;
            }

            // 解析netstat输出格式
            // 示例: "tcp4       0      0  192.168.1.100.53095   142.250.196.142.443  ESTABLISHED"
            // 或UDP: "udp4       0      0 192.168.1.100.5353     224.0.0.251.5353"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let protocol = parts[0].to_string();
                let local_address = parts[3].to_string();
                let remote_address = parts[4].to_string();
                let state = if parts.len() > 5 { parts[5].to_string() } else { "UNKNOWN".to_string() };

                // 更新协议计数
                *protocol_counts.entry(protocol.clone()).or_insert(0) += 1;
                
                // 更新状态计数
                *state_counts.entry(state.clone()).or_insert(0) += 1;

                let connection = NetworkConnection {
                    protocol,
                    local_address,
                    remote_address,
                    state,
                    pid: None,
                    process_name: None,
                    extra: HashMap::new(),
                };

                connections.push(connection);
            }
        }

        let total_count = connections.len();
        let connection_list = NetworkConnectionList {
            connections,
            total_count,
            protocol_counts,
            state_counts,
            system_info: HashMap::new(),
        };

        Ok(Box::new(connection_list))
    }

    /// 解析ifconfig命令输出
    fn parse_ifconfig_output(&self, data: &str) -> ParseResult<Box<dyn ParsedData>> {
        let lines: Vec<&str> = data.lines().collect();
        if lines.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let mut interfaces = Vec::new();
        let mut current_interface: Option<NetworkInterface> = None;
        let mut total_rx_bytes = 0;
        let mut total_tx_bytes = 0;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if let Some(interface) = current_interface.take() {
                    total_rx_bytes += interface.rx_bytes;
                    total_tx_bytes += interface.tx_bytes;
                    interfaces.push(interface);
                }
                continue;
            }

            // 检查是否是接口名称行
            if trimmed.ends_with(':') && !trimmed.starts_with(' ') {
                if let Some(interface) = current_interface.take() {
                    total_rx_bytes += interface.rx_bytes;
                    total_tx_bytes += interface.tx_bytes;
                    interfaces.push(interface);
                }

                let name = trimmed.trim_end_matches(':').to_string();
                current_interface = Some(NetworkInterface {
                    name,
                    status: "UP".to_string(), // 默认状态
                    mtu: 1500, // 默认MTU
                    rx_bytes: 0,
                    rx_packets: 0,
                    rx_errors: 0,
                    tx_bytes: 0,
                    tx_packets: 0,
                    tx_errors: 0,
                    ip_address: None,
                    mac_address: None,
                    extra: HashMap::new(),
                });
                continue;
            }

            // 解析接口属性
            if let Some(ref mut interface) = current_interface {
                if trimmed.starts_with("inet ") {
                    // IP地址
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() > 1 {
                        interface.ip_address = Some(parts[1].to_string());
                    }
                } else if trimmed.starts_with("ether ") {
                    // MAC地址
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() > 1 {
                        interface.mac_address = Some(parts[1].to_string());
                    }
                } else if trimmed.contains("RX packets") {
                    // 接收统计
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if let Some(idx) = parts.iter().position(|p| *p == "packets:")
                        && idx + 1 < parts.len()
                        && let Ok(packets) = parts[idx + 1].parse::<u64>() {
                        interface.rx_packets = packets;
                    }
                    if let Some(idx) = parts.iter().position(|p| *p == "bytes:")
                        && idx + 1 < parts.len()
                        && let Ok(bytes) = parts[idx + 1].parse::<u64>() {
                        interface.rx_bytes = bytes;
                    }
                } else if trimmed.contains("TX packets") {
                    // 发送统计
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if let Some(idx) = parts.iter().position(|p| *p == "packets:")
                        && idx + 1 < parts.len()
                        && let Ok(packets) = parts[idx + 1].parse::<u64>() {
                        interface.tx_packets = packets;
                    }
                    if let Some(idx) = parts.iter().position(|p| *p == "bytes:")
                        && idx + 1 < parts.len()
                        && let Ok(bytes) = parts[idx + 1].parse::<u64>() {
                        interface.tx_bytes = bytes;
                    }
                }
            }
        }

        // 添加最后一个接口
        if let Some(interface) = current_interface.take() {
            total_rx_bytes += interface.rx_bytes;
            total_tx_bytes += interface.tx_bytes;
            interfaces.push(interface);
        }

        let total_count = interfaces.len();
        let interface_list = NetworkInterfaceList {
            interfaces,
            total_count,
            total_rx_bytes,
            total_tx_bytes,
            system_info: HashMap::new(),
        };

        Ok(Box::new(interface_list))
    }

    /// 解析route命令输出
    fn parse_route_output(&self, data: &str) -> ParseResult<Box<dyn ParsedData>> {
        let lines: Vec<&str> = data.lines().collect();
        if lines.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let mut routes = Vec::new();
        let mut default_gateway = None;

        // 跳过标题行
        let start_idx = lines.iter()
            .position(|line| line.contains("Destination") || line.contains("Target"))
            .unwrap_or(0);

        for line in lines.iter().skip(start_idx + 1) {
            if line.trim().is_empty() {
                continue;
            }

            // 解析route输出格式
            // 示例: "default            192.168.1.1        UGSc           en0"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let destination = parts[0].to_string();
                let gateway = parts[1].to_string();
                let flags = parts[2].to_string();
                let interface = parts[3].to_string();

                // 检查是否是默认路由
                if destination == "default" {
                    default_gateway = Some(gateway.clone());
                }

                let route = RouteEntry {
                    destination,
                    gateway,
                    netmask: "0.0.0.0".to_string(), // 默认值
                    interface,
                    flags,
                    metric: None,
                    extra: HashMap::new(),
                };

                routes.push(route);
            }
        }

        let total_count = routes.len();
        let routing_table = RoutingTable {
            routes,
            total_count,
            default_gateway,
            system_info: HashMap::new(),
        };

        Ok(Box::new(routing_table))
    }
}

impl ParserTrait for NetworkParser {
    fn parse(&self, input: &str) -> ParseResult<Box<dyn ParsedData>> {
        // 尝试确定输入格式并选择相应的解析方法
        if input.contains("Proto") && input.contains("Local Address") {
            // 可能是netstat命令输出
            self.parse_netstat_output(input)
        } else if input.contains("flags=") || input.contains("mtu") {
            // 可能是ifconfig命令输出
            self.parse_ifconfig_output(input)
        } else if input.contains("Destination") && input.contains("Gateway") {
            // 可能是route命令输出
            self.parse_route_output(input)
        } else {
            Err(ParseError::InvalidFormat(
                "Unable to determine network data format".to_string(),
            ))
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn can_parse(&self, input: &str) -> bool {
        // 检查输入是否包含网络数据的特征
        input.contains("Proto") || 
        input.contains("Local Address") ||
        input.contains("flags=") ||
        input.contains("mtu") ||
        input.contains("Destination") ||
        input.contains("Gateway") ||
        (input.lines().any(|line| line.split_whitespace().count() > 3) && 
         (input.contains("tcp") || input.contains("udp") || input.contains("inet")))
    }
}

impl ParsedData for NetworkConnectionList {
    fn data_type(&self) -> &str {
        "network_connection_list"
    }

    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }
}

impl ParsedData for NetworkInterfaceList {
    fn data_type(&self) -> &str {
        "network_interface_list"
    }

    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }
}

impl ParsedData for RoutingTable {
    fn data_type(&self) -> &str {
        "routing_table"
    }

    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_parser_creation() {
        let parser = NetworkParser::new();
        assert_eq!(parser.name(), "network");
    }

    #[test]
    fn test_can_parse() {
        let parser = NetworkParser::new();
        
        // 测试netstat输出
        let netstat_output = "Proto Recv-Q Send-Q Local Address           Foreign Address         State\n\
                              tcp4       0      0 192.168.1.100.53095    142.250.196.142.443    ESTABLISHED";
        assert!(parser.can_parse(netstat_output));
        
        // 测试ifconfig输出
        let ifconfig_output = "en0: flags=8863<UP,BROADCAST,SMART,RUNNING,SIMPLEX,MULTICAST> mtu 1500\n\
                               ether 00:11:22:33:44:55 \n\
                               inet 192.168.1.100 netmask 0xffffff00 broadcast 192.168.1.255\n\
                               media: autoselect\n\
                               status: active";
        assert!(parser.can_parse(ifconfig_output));
        
        // 测试route输出
        let route_output = "Destination        Gateway            Flags        Refs      Use   Netif Expire\n\
                            default            192.168.1.1        UGSc          110        0     en0     ";
        assert!(parser.can_parse(route_output));
        
        // 测试无效数据
        let invalid_data = "This is not network data";
        assert!(!parser.can_parse(invalid_data));
    }

    #[test]
    fn test_parse_netstat_output() {
        let parser = NetworkParser::new();
        let netstat_output = "Proto Recv-Q Send-Q Local Address           Foreign Address         State\n\
                              tcp4       0      0 192.168.1.100.53095    142.250.196.142.443    ESTABLISHED\n\
                              tcp4       0      0 192.168.1.100.53096    172.217.164.78.443      ESTABLISHED\n\
                              udp4       0      0 192.168.1.100.5353     224.0.0.251.5353        ";
        
        let result = parser.parse_netstat_output(netstat_output);
        assert!(result.is_ok());
        
        let connection_list = result.unwrap();
        let connection_list = connection_list.as_any_ref().downcast_ref::<NetworkConnectionList>().unwrap();
        
        assert_eq!(connection_list.total_count, 3);
        assert_eq!(connection_list.connections[0].protocol, "tcp4");
        assert_eq!(connection_list.connections[0].state, "ESTABLISHED");
        assert_eq!(connection_list.connections[2].protocol, "udp4");
        
        // 检查协议计数
        assert_eq!(connection_list.protocol_counts.get("tcp4"), Some(&2));
        assert_eq!(connection_list.protocol_counts.get("udp4"), Some(&1));
        
        // 检查状态计数
        assert_eq!(connection_list.state_counts.get("ESTABLISHED"), Some(&2));
    }

    #[test]
    fn test_network_connection_list_data_type() {
        let connections = vec![NetworkConnection {
            protocol: "tcp4".to_string(),
            local_address: "192.168.1.100.53095".to_string(),
            remote_address: "142.250.196.142.443".to_string(),
            state: "ESTABLISHED".to_string(),
            pid: None,
            process_name: None,
            extra: HashMap::new(),
        }];
        
        let connection_list = NetworkConnectionList {
            connections,
            total_count: 1,
            protocol_counts: HashMap::new(),
            state_counts: HashMap::new(),
            system_info: HashMap::new(),
        };
        
        assert_eq!(connection_list.data_type(), "network_connection_list");
        
        let json = connection_list.to_json();
        assert!(json.is_ok());
        assert!(json.unwrap().contains("\"total_count\":1"));
    }
}