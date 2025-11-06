use crate::chat_history::ChatMessage;
use crate::character_storage::{CharacterData, CharacterBook};
use crate::character_session::{ContextBuilderOptions, TokenBudget};
use crate::token_counter::get_token_counter;
use serde::{Deserialize, Serialize};

/// OpenAI 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
    pub name: Option<String>,
    pub tool_calls: Option<Vec<crate::chat_history::ToolCall>>,
    pub tool_call_id: Option<String>,
}

/// 消息类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    System,
    User,
    Assistant,
    Tool,
}

/// 处理后的世界书条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedWorldBookEntry {
    /// 条目的原始数据
    pub entry: serde_json::Value,
    /// 条目的 Token 数量
    pub token_count: usize,
    /// 条目重要性评分
    pub importance_score: f64,
}

/// Token 分配详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAllocation {
    /// 角色核心信息
    pub character: usize,
    /// 世界书条目
    pub worldbook: usize,
    /// 系统指令和工具
    pub system: usize,
    /// 聊天历史
    pub history: usize,
}

/// 构建完成的上下文结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltContextResult {
    /// System 消息数组
    pub system_messages: Vec<OpenAIMessage>,
    /// Assistant 消息数组
    pub assistant_messages: Vec<OpenAIMessage>,
    /// 聊天历史消息
    pub history_messages: Vec<OpenAIMessage>,
    /// 当前用户消息
    pub current_user_message: Option<OpenAIMessage>,
    /// 总 Token 数量
    pub total_tokens: usize,
    /// Token 分配详情
    pub token_allocation: TokenAllocation,
    /// 是否使用了截断
    pub was_truncated: bool,
}

/// 上下文构建器 - 负责构建完整的 AI 对话上下文
pub struct ContextBuilder {
    token_budget: TokenBudget,
    options: ContextBuilderOptions,
}

impl ContextBuilder {
    /// 创建新的上下文构建器
    pub fn new(options: ContextBuilderOptions) -> Self {
        let token_budget = TokenBudget::default();
        Self {
            token_budget,
            options,
        }
    }

    /// 构建完整的对话上下文
    pub fn build_full_context(
        &self,
        character_data: &CharacterData,
        chat_history: &[ChatMessage],
        current_user_message: Option<&str>,
    ) -> Result<BuiltContextResult, String> {
        // 1. 构建 System 消息
        let system_messages = self.build_system_messages(character_data)?;
        let system_tokens = self.count_messages_tokens(&system_messages);

        // 2. 构建 Assistant 消息（角色信息 + 世界书）
        let (assistant_messages, character_tokens, worldbook_tokens) =
            self.build_assistant_messages(character_data)?;

        // 3. 处理聊天历史
        let history_messages = self.build_history_messages(
            chat_history,
            self.token_budget.history_reserved,
        )?;
        let history_tokens = self.count_messages_tokens(&history_messages);

        // 4. 处理当前用户消息
        let current_message = if let Some(content) = current_user_message {
            Some(OpenAIMessage {
                role: "user".to_string(),
                content: content.to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            })
        } else {
            None
        };

        let current_tokens = current_message.as_ref()
            .map(|msg| self.count_message_tokens(msg))
            .unwrap_or(0);

        // 5. 计算 Token 分配
        let token_allocation = TokenAllocation {
            character: character_tokens,
            worldbook: worldbook_tokens,
            system: system_tokens,
            history: history_tokens,
        };

        let total_tokens = system_tokens + character_tokens + worldbook_tokens + history_tokens + current_tokens;
        let was_truncated = total_tokens > self.options.token_limit;

        Ok(BuiltContextResult {
            system_messages,
            assistant_messages,
            history_messages,
            current_user_message: current_message,
            total_tokens,
            token_allocation,
            was_truncated,
        })
    }

    /// 构建 System 消息（包含 role、task、tools、instructions）
    fn build_system_messages(&self, character_data: &CharacterData) -> Result<Vec<OpenAIMessage>, String> {
        let mut content = String::new();

        // 添加 AI 角色定义
        let role = self.process_placeholders(&self.options.ai_role, character_data);
        content.push_str(&format!("role: {}\n", role));

        // 添加 AI 任务定义
        let task = self.process_placeholders(&self.options.ai_task, character_data);
        content.push_str(&format!("task: {}\n", task));

        // 添加工具定义
        content.push_str("tools:\n");
        content.push_str("  - name: \"edit_character\"\n");
        content.push_str("    description: \"编辑角色字段\"\n");
        content.push_str("    parameters: {\"type\": \"object\", \"properties\": {\"field\": {\"type\": \"string\"}, \"value\": {\"type\": \"string\"}}}\n");

        content.push_str("  - name: \"create_worldbook_entry\"\n");
        content.push_str("    description: \"创建世界书条目\"\n");
        content.push_str("    parameters: {\"type\": \"object\", \"properties\": {\"name\": {\"type\": \"string\"}, \"content\": {\"type\": \"string\"}, \"keys\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}}}\n");

        // 添加指令
        content.push_str("instructions: |\n");
        content.push_str("  基于用户需求分析现有角色设定，提供建议并调用相应工具。\n");
        content.push_str("  始终保持角色设定的一致性和逻辑性，遵循用户的具体要求。\n");
        content.push_str("  如果需要修改角色信息，请使用 edit_character 工具。\n");
        content.push_str("  如果需要添加世界书条目，请使用 create_worldbook_entry 工具。\n");

        Ok(vec![OpenAIMessage {
            role: "system".to_string(),
            content,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }])
    }

    /// 构建 Assistant 消息（角色信息 + 世界书）
    fn build_assistant_messages(&self, character_data: &CharacterData) -> Result<(Vec<OpenAIMessage>, usize, usize), String> {
        let mut messages = Vec::new();

        // 1. 构建角色信息消息
        let character_content = self.build_character_content(character_data)?;
        let character_tokens = self.count_tokens(&character_content);

        messages.push(OpenAIMessage {
            role: "assistant".to_string(),
            content: format!("character:\n{}", character_content),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        });

        // 2. 构建世界书消息（如果存在）
        let (_worldbook_content, worldbook_tokens) = if let Some(character_book) = &character_data.card.data.character_book {
            let worldbook_content = self.build_worldbook_content(character_book)?;
            let worldbook_tokens = self.count_tokens(&worldbook_content);

            messages.push(OpenAIMessage {
                role: "assistant".to_string(),
                content: format!("worldbook:\n{}", worldbook_content),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            });

            (worldbook_content, worldbook_tokens)
        } else {
            (String::new(), 0)
        };

        Ok((messages, character_tokens, worldbook_tokens))
    }

    /// 构建角色内容
    fn build_character_content(&self, character_data: &CharacterData) -> Result<String, String> {
        let card_data = &character_data.card.data;
        let mut content = String::new();

        // 基本角色信息
        if !card_data.name.is_empty() {
            content.push_str(&format!("  name: \"{}\"\n", card_data.name));
        }
        if !card_data.description.is_empty() {
            content.push_str(&format!("  description: \"{}\"\n", card_data.description));
        }
        if !card_data.personality.is_empty() {
            content.push_str(&format!("  personality: \"{}\"\n", card_data.personality));
        }
        if !card_data.scenario.is_empty() {
            content.push_str(&format!("  scenario: \"{}\"\n", card_data.scenario));
        }
        if !card_data.first_mes.is_empty() {
            content.push_str(&format!("  first_mes: \"{}\"\n", card_data.first_mes));
        }
        if !card_data.mes_example.is_empty() {
            content.push_str(&format!("  mes_example: \"{}\"\n", card_data.mes_example));
        }
        if !card_data.creator_notes.is_empty() {
            content.push_str(&format!("  creator_notes: \"{}\"\n", card_data.creator_notes));
        }
        if !card_data.system_prompt.is_empty() {
            content.push_str(&format!("  system_prompt: \"{}\"\n", card_data.system_prompt));
        }
        if !card_data.post_history_instructions.is_empty() {
            content.push_str(&format!("  post_history_instructions: \"{}\"\n", card_data.post_history_instructions));
        }

        // 标签
        if !card_data.tags.is_empty() {
            content.push_str(&format!("  tags: [{}]\n",
                card_data.tags.iter()
                    .map(|tag| format!("\"{}\"", tag))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // 创建者和版本
        if !card_data.creator.is_empty() {
            content.push_str(&format!("  creator: \"{}\"\n", card_data.creator));
        }
        if !card_data.character_version.is_empty() {
            content.push_str(&format!("  character_version: \"{}\"\n", card_data.character_version));
        }

        Ok(content)
    }

    /// 构建世界书内容
    fn build_worldbook_content(&self, character_book: &CharacterBook) -> Result<String, String> {
        let mut content = String::new();

        // 世界书基本信息
        if let Some(name) = &character_book.name {
            content.push_str(&format!("  name: \"{}\"\n", name));
        }
        if let Some(description) = &character_book.description {
            content.push_str(&format!("  description: \"{}\"\n", description));
        }
        if let Some(scan_depth) = character_book.scan_depth {
            content.push_str(&format!("  scan_depth: {}\n", scan_depth));
        }
        if let Some(token_budget) = character_book.token_budget {
            content.push_str(&format!("  token_budget: {}\n", token_budget));
        }
        if let Some(recursive_scanning) = character_book.recursive_scanning {
            content.push_str(&format!("  recursive_scanning: {}\n", recursive_scanning));
        }

        // 条目总数
        content.push_str(&format!("  total_entries: {}\n", character_book.entries.len()));

        // 条目内容（按重要性排序）
        content.push_str("  entries:\n");
        let mut processed_entries = Vec::new();

        for (index, entry) in character_book.entries.iter().enumerate() {
            let entry_json = serde_json::to_value(entry).map_err(|e| format!("序列化条目失败: {}", e))?;
            let entry_obj = entry_json.as_object().ok_or("条目不是对象类型")?;
            let entry_content = self.serialize_worldbook_entry(entry_obj, index)?;
            let token_count = self.count_tokens(&entry_content);
            let importance_score = self.calculate_entry_importance(entry_obj);

            processed_entries.push(ProcessedWorldBookEntry {
                entry: entry_json,
                token_count,
                importance_score,
            });
        }

        // 按重要性排序
        processed_entries.sort_by(|a, b| b.importance_score.partial_cmp(&a.importance_score).unwrap());

        // 输出条目（考虑 Token 限制）
        let mut used_tokens = 0;
        for processed_entry in processed_entries {
            if used_tokens + processed_entry.token_count <= self.token_budget.worldbook_reserved {
                let entry_content = self.serialize_worldbook_entry(
                    processed_entry.entry.as_object().unwrap(),
                    0 // index 在这里不重要
                )?;
                content.push_str(&entry_content);
                used_tokens += processed_entry.token_count;
            }
        }

        Ok(content)
    }

    /// 序列化世界书条目
    fn serialize_worldbook_entry(&self, entry: &serde_json::Map<String, serde_json::Value>, _index: usize) -> Result<String, String> {
        let mut content = String::new();
        content.push_str("    - {\n");

        // 基本信息
        if let Some(id) = entry.get("id") {
            content.push_str(&format!("      id: {},\n", id));
        }
        if let Some(name) = entry.get("name").and_then(|v| v.as_str()) {
            content.push_str(&format!("      name: \"{}\",\n", name));
        }
        if let Some(keys) = entry.get("keys").and_then(|v| v.as_array()) {
            content.push_str("      keys: [");
            content.push_str(&keys.iter()
                .filter_map(|v| v.as_str())
                .map(|key| format!("\"{}\"", key))
                .collect::<Vec<_>>()
                .join(", ")
            );
            content.push_str("],\n");
        }
        if let Some(content_text) = entry.get("content").and_then(|v| v.as_str()) {
            content.push_str(&format!("      content: \"{}\",\n", content_text));
        }
        if let Some(enabled) = entry.get("enabled").and_then(|v| v.as_bool()) {
            content.push_str(&format!("      enabled: {},\n", enabled));
        }
        if let Some(priority) = entry.get("priority").and_then(|v| v.as_u64()) {
            content.push_str(&format!("      priority: {},\n", priority));
        }

        content.push_str("    }\n");
        Ok(content)
    }

    /// 计算条目重要性
    fn calculate_entry_importance(&self, entry: &serde_json::Map<String, serde_json::Value>) -> f64 {
        let mut score = 1.0;

        // 启用状态权重
        if entry.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true) {
            score += 2.0;
        }

        // 优先级权重
        if let Some(priority) = entry.get("priority").and_then(|v| v.as_u64()) {
            score += priority as f64 * 0.5;
        }

        // 关键词数量权重
        if let Some(keys) = entry.get("keys").and_then(|v| v.as_array()) {
            score += keys.len() as f64 * 0.3;
        }

        // 内容长度权重（适度）
        if let Some(content) = entry.get("content").and_then(|v| v.as_str()) {
            let word_count = content.split_whitespace().count();
            if word_count > 0 {
                score += (word_count as f64).log10() * 0.2;
            }
        }

        score
    }

    /// 构建历史消息（智能裁剪）
    fn build_history_messages(&self, chat_history: &[ChatMessage], token_limit: usize) -> Result<Vec<OpenAIMessage>, String> {
        let mut messages = Vec::new();
        let mut used_tokens = 0;

        // 从最新消息开始，倒序添加
        for message in chat_history.iter().rev() {
            let openai_message = OpenAIMessage {
                role: message.role.clone(),
                content: message.content.clone(),
                name: message.name.clone(),
                tool_calls: message.tool_calls.clone(),
                tool_call_id: message.tool_call_id.clone(),
            };

            let message_tokens = self.count_message_tokens(&openai_message);

            if used_tokens + message_tokens <= token_limit {
                messages.insert(0, openai_message);
                used_tokens += message_tokens;
            } else {
                break;
            }
        }

        Ok(messages)
    }

    /// 处理占位符替换
    fn process_placeholders(&self, template: &str, character_data: &CharacterData) -> String {
        let mut result = template.to_string();

        // 替换基本占位符
        if let Some(role) = self.options.placeholders.get("{{ROLE}}") {
            result = result.replace("{{ROLE}}", role);
        }
        if let Some(task) = self.options.placeholders.get("{{TASK}}") {
            result = result.replace("{{TASK}}", task);
        }

        // 替换角色相关占位符
        result = result.replace("{{CHARACTER_NAME}}", &character_data.card.data.name);

        result
    }

    /// 计算 Token 数量
    fn count_tokens(&self, text: &str) -> usize {
        let counter = get_token_counter();
        counter.count_tokens(text).token_count
    }

    /// 计算消息的 Token 数量
    fn count_message_tokens(&self, message: &OpenAIMessage) -> usize {
        let counter = get_token_counter();
        let content = serde_json::to_string(message).unwrap_or_default();
        counter.count_tokens(&content).token_count
    }

    /// 计算多个消息的 Token 数量
    fn count_messages_tokens(&self, messages: &[OpenAIMessage]) -> usize {
        messages.iter().map(|msg| self.count_message_tokens(msg)).sum()
    }
}

// ====================== 辅助函数 ======================

/// 创建默认的上下文构建器
pub fn create_default_context_builder() -> ContextBuilder {
    ContextBuilder::new(ContextBuilderOptions::default())
}

/// 创建自定义配置的上下文构建器
pub fn create_context_builder(options: ContextBuilderOptions) -> ContextBuilder {
    ContextBuilder::new(options)
}

// ====================== Tauri命令 ======================

/// 构建上下文（用于测试）
#[tauri::command]
pub async fn build_context(
    _character_uuid: String,
    _token_limit: Option<usize>,
) -> Result<BuiltContextResult, String> {
    // TODO: 在任务1.3中实现完整的会话集成
    Err("build_context 命令将在后续任务中完整实现".to_string())
}