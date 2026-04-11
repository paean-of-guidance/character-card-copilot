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
    /// AI 指令定义（支持占位符）
    pub ai_instructions: String,
    /// 是否在上下文中声明工具
    pub tools_enabled: bool,
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
                  "帮助用户创作和完善角色设定, 需要从多个角度(角色动机，角色心理，角色性格，角色背景)等分析，完成角色卡。当需要局部修改某个字段中的一句话、某个 trait 或某段内容时，优先先读后写：不确定当前文本时使用 read_character_field 或 patch_character_field(dry_run=true) 预览，确认唯一命中后再执行 patch_character_field；只有当用户明确要求重写整个字段时，才使用 edit_character。当处理世界书时，先使用 list_world_book_entries 查看候选，必要时用 read_world_book_entry 读取完整条目；创建使用 create_world_book_entry，更新使用 update_world_book_entry，删除使用 delete_world_book_entry，并尽量传 entry_id 以避免误操作。".to_string(),
              );
        placeholders.insert(
            "{{INSTRUCTIONS}}".to_string(),
            "基于用户需求分析现有角色设定，提供建议并调用相应工具。始终保持角色设定的一致性和逻辑性，遵循用户的具体要求。".to_string(),
        );

        Self {
            token_limit: 102400,
            enable_smart_truncation: true,
            ai_role: "{{ROLE}}".to_string(),
            ai_task: "{{TASK}}".to_string(),
            ai_instructions: "{{INSTRUCTIONS}}".to_string(),
            tools_enabled: true,
            prioritize_chat_history: true,
            placeholders,
        }
    }
}
