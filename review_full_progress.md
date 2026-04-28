# Review Fix Progress

## Base

- Base revision: `mltmyvsy` / `bb0b66d4` (`chore(review): isolate reviewed backend changes`)
- Source reviews: `review/codex`, `review/deepseek`
- Repair line: jj workspace `review-spaghetti-fixes`, bookmark `codex/review-spaghetti-fixes`
- Goal: fix high-value review findings without introducing spaghetti code.

## Findings

| ID | Priority | Files | Source | Decision | Status | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| F01 | High | `character_session.rs`, `session_service.rs` | Codex | Fix | Done | Added session mutation helpers and moved short edit/delete/send mutations onto them. AI generation still uses a snapshot across await boundaries. |
| F02 | High | `AIPanel.vue` | Codex | Fix | Done | Preserves `name`, `tool_calls`, `tool_call_id`, and `reasoning_content` on unmount/remount. |
| F03 | High | `Editor.vue` | Codex | Fix | Done | Removed duplicate mounted load and added UUID/load promise guard. |
| F04 | Medium | `chat_history.rs` | Both | Fix | Done | Uses `BufReader::lines()` and tested blank/bad-line handling. |
| F05 | Medium | `config.rs`, `context_builder.rs`, `session_service.rs` | Deepseek | Fix | Done | Token budget is derived from `ContextBuilderOptions.token_limit`; AI requests derive that limit from API `context_window`. |
| F06 | Medium | `tools/*character*`, `tools/world_book_*` | Deepseek | Fix | Done | Added shared character field helpers and shared tool result builders. |
| F07 | Medium | `Editor.vue`, `CharacterEditorPanel.vue`, `stores/character.ts` | Codex | Fix | Done | Added shared frontend formatter for alternate greetings and tags. |
| F08 | High | `ai_chat.rs` | Deepseek | Fix | Done | Split into `types`, `adapter`, `formatting`, and `service` modules with public re-exports. |
| F09 | Medium | `session_service.rs` | Deepseek | Partial Fix | Done | Local risk areas were reduced; broader service decomposition remains optional follow-up. |
| F10 | Low | `character_storage.rs` | Deepseek | Fix | Done | Added an explicit fast path that skips asset migration when `card.png` and `thumbnail.png` are already current. |
| F11 | Low | `context_builder.rs` | Deepseek | Fix | Done | Moved static tool declarations into a `&'static str` constant and removed the unused worldbook content binding. |
| F12 | Low | `world_book_creator.rs` | Deepseek | Fix | Done | Extracted default entry extensions, entry initialization, parameter application, validation, event emission, and result building into focused helpers. |
| F13 | Medium | `events.rs`, `event_bus.rs` | Deepseek | Fix | Done | Removed the pure pass-through `EventBus` facade and switched call sites to `EventEmitter` directly. |
| F14 | Medium | `lib.rs` | Deepseek | False Positive | False Positive | Tauri does not provide the claimed no-cost command auto-discovery replacement. |

## Phases

1. Progress document and branch isolation: Done.
2. Behavior risk fixes: Done.
3. Shared helpers and duplication cleanup: Done.
4. AI chat module split: Done.
5. Checks and handoff notes: Done.
6. Deferred low-risk cleanup pass: Done.
7. Event facade cleanup: Done.

## Handoff Notes

- Keep functions small and named for the behavior they own.
- Avoid adding new cross-module shortcuts; prefer shared helpers in narrowly named modules.
- Do not include `review_result_d.md` from the original `review/deepseek` workspace.

## Verification

- `pnpm check`: passed.
- `pnpm check:web`: passed after EventBus removal.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed without warnings.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed without warnings, 24 tests.
- Removed the stale `genai v0.5.3` crates.io patch so Cargo no longer reports an unused patch.
