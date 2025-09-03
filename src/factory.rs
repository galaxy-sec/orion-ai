use orion_error::UvsConfFrom;

use crate::client::AiClientBuilder;
use crate::error::{AiResult, OrionAiReason};
use crate::provider::AiRequest;
use crate::thread::recorder::ThreadClient;

use super::AiConfig;
use super::client::{AiClient, AiClientTrait, AiCoreClient};
use super::provider::AiResponse;
/// AI客户端枚举，支持静态分发
pub enum AiClientEnum {
    Basic(Box<AiClient>),
    ThreadRecording(Box<ThreadClient>),
}

impl AiClientEnum {
    /// 创建基础AiClient
    pub fn new(config: AiConfig) -> AiResult<Self> {
        Ok(Self::Basic(Box::new(Self::new_core(config)?)))
    }
    fn new_core(config: AiConfig) -> AiResult<AiClient> {
        // 验证配置
        let mut validated_config = config.clone();
        validated_config.validate_and_postprocess().map_err(|e| {
            OrionAiReason::from_conf(format!("Configuration validation failed: {e}"))
        })?;

        AiClientBuilder::new(config).build()
    }

    /// 创建Thread记录客户端
    pub fn new_with_thread_recording(config: AiConfig) -> AiResult<Self> {
        let inner_config = config.clone();
        let basic_client = Self::new_core(inner_config)?;
        let thread_config = config.thread.clone();

        Ok(Self::ThreadRecording(Box::new(ThreadClient::new(
            AiCoreClient::Basic(basic_client),
            thread_config,
        ))))
    }

    /// 根据配置自动选择客户端类型
    pub fn new_auto(config: AiConfig) -> AiResult<Self> {
        // 验证配置
        let mut validated_config = config;
        validated_config.validate_and_postprocess().map_err(|e| {
            OrionAiReason::from_conf(format!("Configuration validation failed: {e}"))
        })?;

        if validated_config.thread.enabled {
            Self::new_with_thread_recording(validated_config)
        } else {
            Self::new(validated_config)
        }
    }

    /// 发送AI请求
    pub async fn send_request(&self, request: AiRequest) -> AiResult<AiResponse> {
        match self {
            Self::Basic(client) => client.send_request(request).await,
            Self::ThreadRecording(client) => client.as_ref().send_request(request).await,
        }
    }
}
