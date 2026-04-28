use crate::api_config::ApiProvider;
use genai::adapter::AdapterKind;

pub(crate) fn adapter_kind(provider: ApiProvider) -> AdapterKind {
    match provider {
        ApiProvider::OpenAiCompatible => AdapterKind::OpenAI,
        ApiProvider::OpenAiResponses => AdapterKind::OpenAI,
        ApiProvider::Claude => AdapterKind::Anthropic,
        ApiProvider::GeminiV1Beta => AdapterKind::Gemini,
    }
}
