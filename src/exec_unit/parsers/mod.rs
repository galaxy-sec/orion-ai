//! 解析器模块
//! 
//! 该模块提供了用于解析系统命令输出的解析器框架，
//! 支持进程、I/O和网络数据的解析。

mod process;
mod network;

use std::fmt;
use thiserror::Error;

/// 解析错误类型
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("输入数据为空")]
    EmptyInput,
    #[error("数据格式无效: {0}")]
    InvalidFormat(String),
    #[error("缺少必需字段: {0}")]
    MissingField(String),
    #[error("数值解析错误: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("浮点数解析错误: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("其他解析错误: {0}")]
    Other(String),
}

/// 解析结果类型
pub type ParseResult<T> = Result<T, ParseError>;

/// 解析器trait
/// 
/// 所有解析器都需要实现此trait，提供统一的解析接口
pub trait Parser<T> {
    /// 解析输入数据并返回解析结果
    fn parse(&self, input: &str) -> ParseResult<T>;
    
    /// 获取解析器名称
    fn name(&self) -> &str;
    
    /// 检查输入数据是否适合此解析器
    fn can_parse(&self, input: &str) -> bool;
}

/// 解析器注册表
/// 
/// 管理所有可用的解析器，提供查找和注册功能
#[derive(Default)]
pub struct ParserRegistry {
    parsers: Vec<Box<dyn ParserTrait>>,
}

/// 解析器trait对象，用于类型擦除
pub trait ParserTrait: fmt::Debug + Send + Sync {
    /// 解析输入数据并返回解析结果
    fn parse(&self, input: &str) -> ParseResult<Box<dyn ParsedData>>;
    
    /// 获取解析器名称
    fn name(&self) -> &str;
    
    /// 检查输入数据是否适合此解析器
    fn can_parse(&self, input: &str) -> bool;
}

/// 解析数据trait
/// 
/// 所有解析结果都需要实现此trait，提供统一的数据接口
pub trait ParsedData: fmt::Debug + Send + Sync {
    /// 获取数据类型名称
    fn data_type(&self) -> &str;
    
    /// 将数据转换为JSON字符串
    fn to_json(&self) -> Result<String, serde_json::Error>;
    
    /// 将数据转换为 Any trait 对象，用于类型转换
    fn as_any_ref(&self) -> &dyn std::any::Any;
}

impl ParserRegistry {
    /// 创建新的解析器注册表
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 注册解析器
    pub fn register<P>(&mut self, parser: P) 
    where 
        P: ParserTrait + 'static,
    {
        self.parsers.push(Box::new(parser));
    }
    
    /// 查找适合解析输入数据的解析器
    pub fn find_parser(&self, input: &str) -> Option<&dyn ParserTrait> {
        self.parsers
            .iter()
            .find(|parser| parser.can_parse(input))
            .map(|parser| parser.as_ref())
    }
    
    /// 使用适合的解析器解析输入数据
    pub fn parse(&self, input: &str) -> ParseResult<Box<dyn ParsedData>> {
        if let Some(parser) = self.find_parser(input) {
            parser.parse(input)
        } else {
            Err(ParseError::InvalidFormat(
                "没有找到适合的解析器".to_string()
            ))
        }
    }
    
    /// 获取所有注册的解析器名称
    pub fn parser_names(&self) -> Vec<&str> {
        self.parsers
            .iter()
            .map(|parser| parser.name())
            .collect()
    }
}

impl fmt::Debug for ParserRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParserRegistry")
            .field("parsers", &self.parser_names())
            .finish()
    }
}

/// 通用解析器实现
/// 
/// 这是一个通用的解析器实现，可以用于简单的解析场景
#[allow(dead_code)]
pub struct GenericParser<F> 
where 
    F: Fn(&str) -> ParseResult<Box<dyn ParsedData>> + Send + Sync,
{
    name: String,
    parse_func: F,
    can_parse_func: Box<dyn Fn(&str) -> bool + Send + Sync>,
}

impl<F> std::fmt::Debug for GenericParser<F> 
where 
    F: Fn(&str) -> ParseResult<Box<dyn ParsedData>> + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericParser")
            .field("name", &self.name)
            .field("parse_func", &"Fn(&str) -> ParseResult<Box<dyn ParsedData>>")
            .field("can_parse_func", &"Box<dyn Fn(&str) -> bool>")
            .finish()
    }
}

impl<F> GenericParser<F> 
where 
    F: Fn(&str) -> ParseResult<Box<dyn ParsedData>> + Send + Sync,
{
    /// 创建新的通用解析器
    #[allow(dead_code)]
    pub fn new<N, C>(name: N, parse_func: F, can_parse_func: C) -> Self
    where
        N: Into<String>,
        C: Fn(&str) -> bool + 'static + Send + Sync,
    {
        Self {
            name: name.into(),
            parse_func,
            can_parse_func: Box::new(can_parse_func),
        }
    }
}

impl<F> ParserTrait for GenericParser<F> 
where 
    F: Fn(&str) -> ParseResult<Box<dyn ParsedData>> + Send + Sync,
{
    fn parse(&self, input: &str) -> ParseResult<Box<dyn ParsedData>> {
        (self.parse_func)(input)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn can_parse(&self, input: &str) -> bool {
        (self.can_parse_func)(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[derive(Debug)]
    struct TestParsedData {
        value: String,
    }
    
    impl ParsedData for TestParsedData {
        fn data_type(&self) -> &str {
            "test"
        }
        
        fn to_json(&self) -> Result<String, serde_json::Error> {
            serde_json::to_string(&json!({
                "type": "test",
                "value": self.value
            }))
        }
        
        fn as_any_ref(&self) -> &dyn std::any::Any {
            self
        }
    }
    
    #[test]
    fn test_parser_registry() {
        let mut registry = ParserRegistry::new();
        
        // 注册测试解析器
        registry.register(GenericParser::new(
            "test_parser",
            |input| {
                Ok(Box::new(TestParsedData {
                    value: input.to_string(),
                }))
            },
            |_input| true,
        ));
        
        // 测试解析
        let result = registry.parse("test input");
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert_eq!(data.data_type(), "test");
        
        // 测试查找解析器
        let parser = registry.find_parser("test input");
        assert!(parser.is_some());
        assert_eq!(parser.unwrap().name(), "test_parser");
    }
    
    #[test]
    fn test_empty_input_error() {
        let mut registry = ParserRegistry::new();
        
        registry.register(GenericParser::new(
            "empty_parser",
            |_input| Err(ParseError::EmptyInput),
            |_input| true,
        ));
        
        let result = registry.parse("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::EmptyInput));
    }
}