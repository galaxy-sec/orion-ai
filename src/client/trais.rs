use crate::error::AiResult;
use crate::provider::{AiRequest, AiResponse};
use crate::roleid::AiRoleID;
use crate::FunctionDefinition;
use async_trait::async_trait;

use super::core::AiClient;

/// AI客户端发送类型枚举
pub enum AiCoreClient {
    Basic(AiClient),
}

/// AI客户端trait定义
#[async_trait]
pub trait AiClientTrait: Send + Sync {
    async fn send_request(&self, request: AiRequest) -> AiResult<AiResponse>;
    async fn smart_role_request(&self, role: &AiRoleID, user_input: &str) -> AiResult<AiResponse>;
    async fn role_funs_request(
        &self,
        role: &AiRoleID,
        user_input: &str,
        func: Vec<FunctionDefinition>,
    ) -> AiResult<AiResponse>;
}

#[async_trait]
impl AiClientTrait for AiCoreClient {
    async fn send_request(&self, request: AiRequest) -> AiResult<AiResponse> {
        match self {
            Self::Basic(o) => o.send_request(request).await,
        }
    }

    async fn smart_role_request(&self, role: &AiRoleID, user_input: &str) -> AiResult<AiResponse> {
        match self {
            Self::Basic(o) => o.smart_role_request(role, user_input).await,
        }
    }
    async fn role_funs_request(
        &self,
        role: &AiRoleID,
        user_input: &str,
        func: Vec<FunctionDefinition>,
    ) -> AiResult<AiResponse> {
        match self {
            Self::Basic(o) => o.role_funs_request(role, user_input, func).await,
        }
    }
}
