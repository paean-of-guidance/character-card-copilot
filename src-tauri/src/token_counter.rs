use serde::{Deserialize, Serialize};
use tiktoken_rs::{cl100k_base, CoreBPE};
use std::collections::HashSet;

/// Token 计数结果
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenCountResult {
    pub text: String,
    pub token_count: usize,
    pub char_count: usize,
}

/// Token 计数服务
pub struct TokenCounter {
    encoding: CoreBPE,
}

impl TokenCounter {
    /// 创建新的 Token 计数器实例
    pub fn new() -> Result<Self, String> {
        let encoding = cl100k_base().map_err(|e| format!("Failed to load tokenizer: {}", e))?;
        Ok(Self { encoding })
    }

    /// 计算单个文本的 Token 数量
    pub fn count_tokens(&self, text: &str) -> TokenCountResult {
        let allowed_special = HashSet::new(); // 不允许任何特殊token
        let (tokens, _token_count) = self.encoding.encode(text, &allowed_special);
        TokenCountResult {
            text: text.to_string(),
            token_count: tokens.len(),
            char_count: text.chars().count(),
        }
    }

    /// 批量计算多个文本的 Token 数量
    pub fn count_tokens_batch(&self, texts: &[String]) -> Vec<TokenCountResult> {
        texts
            .iter()
            .map(|text| self.count_tokens(text))
            .collect()
    }

    /// 检查文本是否超出 Token 限制
    pub fn is_within_limit(&self, text: &str, limit: usize) -> bool {
        let allowed_special = HashSet::new();
        let (tokens, _token_count) = self.encoding.encode(text, &allowed_special);
        tokens.len() <= limit
    }

    /// 截断文本以符合 Token 限制
    pub fn truncate_to_limit(&self, text: &str, limit: usize) -> String {
        let allowed_special = HashSet::new();
        let (tokens, _token_count) = self.encoding.encode(text, &allowed_special);
        if tokens.len() <= limit {
            return text.to_string();
        }

        let truncated_tokens = tokens.into_iter().take(limit).collect::<Vec<_>>();
        self.encoding.decode(truncated_tokens).unwrap_or_else(|_| {
            // 如果解码失败，返回截断的原始文本
            let char_limit = limit * 4; // 粗略估算：1 token ≈ 4 字符
            text.chars().take(char_limit).collect::<String>()
        })
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new().expect("Failed to create TokenCounter")
    }
}

/// 全局 Token 计数器实例
static mut TOKEN_COUNTER: Option<TokenCounter> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// 获取全局 Token 计数器实例
pub fn get_token_counter() -> &'static TokenCounter {
    unsafe {
        INIT.call_once(|| {
            TOKEN_COUNTER = Some(TokenCounter::new().expect("Failed to initialize TokenCounter"));
        });
        TOKEN_COUNTER.as_ref().unwrap()
    }
}