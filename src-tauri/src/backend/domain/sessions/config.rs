use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token 预算分配策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    /// 总限制：102400 (128k * 0.8)
    pub total_limit: usize,
    /// System 消息保留：15%
    pub system_reserved: usize,
    /// 角色核心信息保留：35%
    pub character_reserved: usize,
    /// 世界书条目保留：20%
    pub worldbook_reserved: usize,
    /// 聊天历史保留：30%
    pub history_reserved: usize,
}

impl Default for TokenBudget {
    fn default() -> Self {
        let total = 102400; // 128k * 0.8
        Self {
            total_limit: total,
            system_reserved: (total as f64 * 0.15) as usize,
            character_reserved: (total as f64 * 0.35) as usize,
            worldbook_reserved: (total as f64 * 0.20) as usize,
            history_reserved: (total as f64 * 0.30) as usize,
        }
    }
}

/// 上下文构建配置选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBuilderOptions {
    /// Token 预算限制
    pub token_limit: usize,
    /// 是否启用智能裁剪
    pub enable_smart_truncation: bool,
    /// AI 角色定义（支持占位符）
    pub ai_role: String,
    /// AI 任务定义（支持占位符）
    pub ai_task: String,
    /// 是否优先保留聊天历史
    pub prioritize_chat_history: bool,
    /// 占位符替换映射
    pub placeholders: HashMap<String, String>,
}

impl Default for ContextBuilderOptions {
    fn default() -> Self {
        let mut placeholders = HashMap::new();
        placeholders.insert("{{ROLE}}".to_string(), "角色卡编写助手".to_string());
        placeholders.insert(
            "{{TASK}}".to_string(),
            "帮助用户创作和完善角色设定, 需要从多个角度(角色动机，角色心理，角色性格，角色背景)等分析，完成角色卡。当用户要求(帮忙填写)的时候，主动使用工具(edit_character)填写用户要求的field。当用户确认添加worldbook条目的时候，使用工具(create_world_book_entry)".to_string(),
        );

        Self {
            token_limit: 102400,
            enable_smart_truncation: true,
            ai_role: "{{ROLE}}".to_string(),
            ai_task: "{{TASK}}".to_string(),
            prioritize_chat_history: true,
            placeholders,
        }
    }
}

